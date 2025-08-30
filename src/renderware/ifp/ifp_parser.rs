//! # IFP Parser
//!
//! A parser for RenderWare IFP (Animation Package) files, used for character
//! animations in Grand Theft Auto 3, Vice City, and San Andreas. This module
//! supports both `ANP3` (GTA3/VC) and `ANPK` (SA) formats.
//!
//! ## Features
//!
//! - Parses animation names, bone keyframes, and hierarchy.
//! - Automatically detects and handles `ANP3` and `ANPK` versions.
//! - Deserializes animation data into a structured `RwIfp` format.
//!
//! ## Example
//!
//! ```no_run
//! use rw_parser_rs::renderware::ifp::ifp_parser::IfpParser;
//! use std::fs;
//!
//! let file_data = fs::read("path/to/your/animation.ifp").unwrap();
//! let mut parser = IfpParser::new(&file_data);
//! let ifp_data = parser.parse().unwrap();
//!
//! println!("Animation package name: {}", ifp_data.name);
//! ```

use crate::renderware::rw_file::RwFile;
use crate::renderware::common::types::{RwVector3, RwQuaternion};
use std::io::Result;

use serde::Serialize;

/// Represents the version of the IFP file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IfpVersion {
    /// GTA3 / Vice City format.
    ANP3,
    /// San Andreas format.
    ANPK,
    /// An unsupported or unknown format.
    UNSUPPORTED,
}

/// Represents the top-level structure of a parsed IFP file.
///
/// This struct contains the package name and a list of all animations
/// included in the file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwIfp {
    /// The format version of the IFP file (`ANP3` or `ANPK`).
    pub version: IfpVersion,
    /// The name of the animation package.
    pub name: String,
    /// A list of animations contained within the package.
    pub animations: Vec<RwIfpAnimation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwIfpAnimation {
    pub name: String,
    pub bones: Vec<RwIfpBone>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwIfpBone {
    pub name: String,
    pub keyframe_type: String,
    pub use_bone_id: bool,
    pub bone_id: i32,
    pub keyframes: Vec<RwIfpKeyframe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwIfpKeyframe {
    pub time: f32,
    pub position: RwVector3,
    pub rotation: RwQuaternion,
    pub scale: RwVector3,
}

/// The main parser for IFP files.
///
/// This struct holds the file buffer and provides the `parse` method to
/// deserialize the IFP animation data.
pub struct IfpParser<'a> {
    file: RwFile<'a>,
}

impl<'a> IfpParser<'a> {
    /// Creates a new `IfpParser` instance with the given file buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A byte slice containing the raw IFP file data.
    pub fn new(buffer: &'a [u8]) -> Self {
        IfpParser {
            file: RwFile::new(buffer),
        }
    }

    /// Parses the entire IFP file buffer.
    ///
    /// This method detects the IFP version (`ANP3` or `ANPK`) based on the
    /// file signature and calls the appropriate internal parsing method.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `RwIfp` data or an `std::io::Error`
    /// if the file format is not supported or a parsing error occurs.
    pub fn parse(&mut self) -> Result<RwIfp> {
        let file_signature = self.file.get_stream().read_string(4)?;
        self.file.get_stream().set_position(0);

        match file_signature.as_str() {
            "ANP3" => self.read_anp3(),
            "ANPK" => self.read_anpk(),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported IFP version")),
        }
    }

    fn read_anp3(&mut self) -> Result<RwIfp> {
        self.file.get_stream().skip(4)?; // ANP3
        let _size = self.file.get_stream().read_u32()?;
        let name = self.file.get_stream().read_string(24)?;
        let animations_count = self.file.get_stream().read_u32()?;
        let mut animations = Vec::with_capacity(animations_count as usize);

        for _ in 0..animations_count {
            animations.push(self.read_anp3_animation()?);
        }

        Ok(RwIfp {
            version: IfpVersion::ANP3,
            name,
            animations,
        })
    }

    fn read_anp3_animation(&mut self) -> Result<RwIfpAnimation> {
        let name = self.file.get_stream().read_string(24)?;
        let bones_count = self.file.get_stream().read_u32()?;
        self.file.get_stream().skip(8)?; // keyframes_size, unk
        let mut bones = Vec::with_capacity(bones_count as usize);

        for _ in 0..bones_count {
            bones.push(self.read_anp3_bone()?);
        }

        Ok(RwIfpAnimation { name, bones })
    }

    fn read_anp3_bone(&mut self) -> Result<RwIfpBone> {
        let name = self.file.get_stream().read_string(24)?;
        let keyframe_type_num = self.file.get_stream().read_u32()?;
        let keyframes_count = self.file.get_stream().read_u32()?;
        let keyframe_type = if keyframe_type_num == 4 { "KRT0" } else { "KR00" }.to_string();
        let bone_id = self.file.get_stream().read_i32()?;
        let mut keyframes = Vec::with_capacity(keyframes_count as usize);

        for _ in 0..keyframes_count {
            let qx = self.file.get_stream().read_i16()? as f32 / 4096.0;
            let qy = self.file.get_stream().read_i16()? as f32 / 4096.0;
            let qz = self.file.get_stream().read_i16()? as f32 / 4096.0;
            let qw = self.file.get_stream().read_i16()? as f32 / 4096.0;
            let time = self.file.get_stream().read_i16()? as f32;

            let (px, py, pz) = if keyframe_type.as_bytes()[2] == b'T' {
                (
                    self.file.get_stream().read_i16()? as f32 / 1024.0,
                    self.file.get_stream().read_i16()? as f32 / 1024.0,
                    self.file.get_stream().read_i16()? as f32 / 1024.0,
                )
            } else {
                (0.0, 0.0, 0.0)
            };
            
            keyframes.push(RwIfpKeyframe {
                time,
                position: RwVector3 { x: px, y: py, z: pz },
                rotation: RwQuaternion { w: qw, x: qx, y: qy, z: qz },
                scale: RwVector3 { x: 1.0, y: 1.0, z: 1.0 },
            });
        }

        Ok(RwIfpBone {
            name,
            keyframe_type,
            use_bone_id: true,
            bone_id,
            keyframes,
        })
    }

    fn read_anpk(&mut self) -> Result<RwIfp> {
        self.file.get_stream().skip(4)?; // ANPK
        let _size = self.file.get_stream().read_u32()?;
        self.file.get_stream().skip(4)?; // INFO
        let info_len = self.file.get_stream().read_u32()?;
        let animations_count = self.file.get_stream().read_u32()?;
        let name = self.file.get_stream().read_string((info_len - 4) as usize)?;
        let name_align_len = (4 - info_len % 4) % 4;
        self.file.get_stream().skip(name_align_len as u64)?;

        let mut animations = Vec::with_capacity(animations_count as usize);
        for _ in 0..animations_count {
            animations.push(self.read_anpk_animation()?);
        }

        Ok(RwIfp {
            version: IfpVersion::ANPK,
            name,
            animations,
        })
    }

    fn read_anpk_animation(&mut self) -> Result<RwIfpAnimation> {
        self.file.get_stream().skip(4)?; // NAME
        let name_len = self.file.get_stream().read_u32()?;
        let name = self.file.get_stream().read_string(name_len as usize)?;
        self.file.get_stream().skip(((4 - name_len % 4) % 4) as u64)?;
        self.file.get_stream().skip(16)?; // DGAN, animation_size, INFO, unk_size
        let bones_count = self.file.get_stream().read_u32()?;
        self.file.get_stream().skip(4)?; // unk
        
        let mut bones = Vec::with_capacity(bones_count as usize);
        for _ in 0..bones_count {
            bones.push(self.read_anpk_bone()?);
        }

        Ok(RwIfpAnimation { name, bones })
    }

    fn read_anpk_bone(&mut self) -> Result<RwIfpBone> {
        self.file.get_stream().skip(8)?; // CPAN, bone_len
        self.file.get_stream().skip(4)?; // ANIM
        let anim_len = self.file.get_stream().read_u32()?;
        let name = self.file.get_stream().read_string(28)?;
        let keyframes_count = self.file.get_stream().read_u32()?;
        self.file.get_stream().skip(8)?; // unk

        let use_bone_id = anim_len == 44;
        let bone_id = if use_bone_id {
            self.file.get_stream().read_i32()?
        } else {
            self.file.get_stream().skip(8)?;
            0
        };

        let mut keyframe_type = "K000".to_string();
        let mut keyframes = Vec::new();

        if keyframes_count > 0 {
            keyframe_type = self.file.get_stream().read_string(4)?;
            self.file.get_stream().skip(4)?; // keyframes_len

            for _ in 0..keyframes_count {
                let qx = self.file.get_stream().read_f32()?;
                let qy = self.file.get_stream().read_f32()?;
                let qz = self.file.get_stream().read_f32()?;
                let qw = self.file.get_stream().read_f32()?;

                let (px, py, pz) = if keyframe_type.as_bytes()[2] == b'T' {
                    (self.file.get_stream().read_f32()?, self.file.get_stream().read_f32()?, self.file.get_stream().read_f32()?)
                } else {
                    (0.0, 0.0, 0.0)
                };

                let (sx, sy, sz) = if keyframe_type.as_bytes()[3] == b'S' {
                    (self.file.get_stream().read_f32()?, self.file.get_stream().read_f32()?, self.file.get_stream().read_f32()?)
                } else {
                    (1.0, 1.0, 1.0)
                };

                let time = self.file.get_stream().read_f32()?;

                keyframes.push(RwIfpKeyframe {
                    time,
                    position: RwVector3 { x: px, y: py, z: pz },
                    rotation: RwQuaternion { w: qw, x: qx, y: qy, z: qz },
                    scale: RwVector3 { x: sx, y: sy, z: sz },
                });
            }
        }

        Ok(RwIfpBone {
            name,
            keyframe_type,
            use_bone_id,
            bone_id,
            keyframes,
        })
    }
}