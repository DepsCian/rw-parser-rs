//! # DFF Parser
//!
//! A parser for RenderWare DFF (Clump) files, commonly used in Grand Theft Auto 3,
//! Vice City, and San Andreas. This module provides structures and methods to
//! deserialize binary DFF data into a structured format.
//!
//! ## Features
//!
//! - Parses geometry, materials, frames, and skinning data.
//! - Determines the model type (Generic, Vehicle, Skin).
//! - Supports multiple RenderWare versions.
//! - Outputs a serializable `RwDff` structure.
//!
//! ## Example
//!
//! ```no_run
//! use rw_parser_rs::renderware::dff::dff_parser::DffParser;
//! use std::fs;
//!
//! let file_data = fs::read("path/to/your/model.dff").unwrap();
//! let mut parser = DffParser::new(&file_data);
//! let dff_data = parser.parse().unwrap();
//!
//! println!("Model version: {}", dff_data.version);
//! ```

use super::dff_model_type::DffModelType;
use crate::renderware::common::types::{
    RwColor, RwMatrix3, RwMatrix4, RwSphere, RwTextureCoordinate, RwTriangle, RwVector3, RwVector4,
};
use crate::renderware::rw_file::RwFile;
use crate::renderware::rw_sections::RwSections;
use crate::utils::rw_version::{unpack_version, RwVersion};
use std::io::Result;
use num::FromPrimitive;

use serde::Serialize;

/// Represents the top-level structure of a parsed DFF file.
///
/// This struct contains all the deserialized data from a RenderWare Clump,
/// including geometry, frame hierarchy, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwDff {
    /// The determined type of the model (e.g., Skin, Vehicle).
    pub model_type: DffModelType,
    /// The RenderWare version string (e.g., "3.6.0.3").
    pub version: String,
    /// The packed integer representation of the RenderWare version.
    pub version_number: u32,
    /// A list of geometries contained within the DFF file.
    pub geometry_list: Option<RwGeometryList>,
    /// The frame hierarchy (skeleton) of the model.
    pub frame_list: Option<RwFrameList>,
    /// Atomic data mapping geometries to frames.
    pub atomics: Vec<u32>,
    /// A list of dummy object names.
    pub dummies: Vec<String>,
    /// Animation node data, typically for skinned models.
    pub anim_nodes: Vec<RwAnimNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwClump {
    pub atomic_count: u32,
    pub light_count: Option<u32>,
    pub camera_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwAnimNode {
    pub bone_id: i32,
    pub bones_count: i32,
    pub bones: Vec<RwBone>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RwBone {
    pub bone_id: i32,
    pub bone_index: i32,
    pub flags: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwFrame {
    pub rotation_matrix: RwMatrix3,
    pub coordinates_offset: RwVector3,
    pub parent_frame: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwFrameList {
    pub frame_count: u32,
    pub frames: Vec<RwFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RwTexture {
    pub texture_filtering: u8,
    pub u_addressing: u8,
    pub v_addressing: u8,
    pub uses_mip_levels: bool,
    pub texture_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwMaterial {
    pub color: RwColor,
    pub is_textured: bool,
    pub ambient: Option<f32>,
    pub specular: Option<f32>,
    pub diffuse: Option<f32>,
    pub texture: Option<RwTexture>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwMaterialList {
    pub material_instance_count: u32,
    pub material_data: Vec<RwMaterial>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwGeometry {
    pub vertex_color_information: Vec<RwColor>,
    pub texture_coordinates_count: u8,
    pub texture_mapping_information: Vec<Vec<RwTextureCoordinate>>,
    pub has_vertices: bool,
    pub has_normals: bool,
    pub triangle_information: Vec<RwTriangle>,
    pub vertex_information: Vec<RwVector3>,
    pub normal_information: Vec<RwVector3>,
    pub bounding_sphere: Option<RwSphere>,
    pub material_list: RwMaterialList,
    pub bin_mesh: RwBinMesh,
    pub skin: Option<RwSkin>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwGeometryList {
    pub geometric_object_count: u32,
    pub geometries: Vec<RwGeometry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RwAtomic {
    pub frame_index: u32,
    pub geometry_index: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwBinMesh {
    pub mesh_count: u32,
    pub meshes: Vec<RwMesh>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwSkin {
    pub bone_count: u8,
    pub used_bone_count: u8,
    pub max_weights_per_vertex: u8,
    pub bone_vertex_indices: Vec<Vec<u8>>,
    pub vertex_weights: Vec<Vec<f32>>,
    pub inverse_bone_matrices: Vec<RwMatrix4>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwMesh {
    pub material_index: u32,
    pub index_count: u32,
    pub indices: Vec<u32>,
}

/// The main parser for DFF files.
///
/// This struct holds the file buffer and provides the `parse` method to
/// deserialize the DFF data.
pub struct DffParser<'a> {
    file: RwFile<'a>,
}

impl<'a> DffParser<'a> {
    /// Creates a new `DffParser` instance with the given file buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A byte slice containing the raw DFF file data.
    pub fn new(buffer: &'a [u8]) -> Self {
        DffParser {
            file: RwFile::new(buffer),
        }
    }

    /// Parses the entire DFF file buffer.
    ///
    /// This method iterates through the RenderWare sections in the file,
    /// deserializing each part and assembling the final `RwDff` struct.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `RwDff` data or an `std::io::Error`
    /// if the parsing fails.
    pub fn parse(&mut self) -> Result<RwDff> {
        let mut version: Option<String> = None;
        let mut version_number: Option<u32> = None;
        let mut atomics: Vec<u32> = Vec::new();
        let mut dummies: Vec<String> = Vec::new();
        let mut anim_nodes: Vec<RwAnimNode> = Vec::new();
        let mut geometry_list: Option<RwGeometryList> = None;
        let mut frame_list: Option<RwFrameList> = None;

        while self.file.get_stream().get_position() < self.file.get_stream().get_size() {
            let header = self.file.read_section_header()?;

            if header.section_type == 0 {
                break;
            }

            if header.section_size == 0 {
                continue;
            }

            let section_type_enum: Option<RwSections> = FromPrimitive::from_u32(header.section_type);

            match section_type_enum {
                Some(RwSections::RwClump) => {
                    version_number = Some(unpack_version(header.version_number));
                    version = Some(RwVersion::new().get_version_string(version_number.unwrap()).unwrap_or_default());
                }
                Some(RwSections::RwFrameList) => {
                    frame_list = Some(self.read_frame_list()?);
                }
                Some(RwSections::RwExtension) => {
                    let extension_header = self.file.read_section_header()?;
                    let extension_type_enum: Option<RwSections> = FromPrimitive::from_u32(extension_header.section_type);
                    match extension_type_enum {
                        Some(RwSections::RwNodeName) => {
                            dummies.push(self.file.get_stream().read_string(extension_header.section_size as usize)?);
                        }
                        Some(RwSections::RwAnim) => {
                            anim_nodes.push(self.read_anim_node()?);
                        }
                        _ => {
                            self.file.get_stream().skip(extension_header.section_size as u64)?;
                        }
                    }
                }
                Some(RwSections::RwGeometryList) => {
                    geometry_list = Some(self.read_geometry_list()?);
                }
                Some(RwSections::RwAtomic) => {
                    let atomic = self.read_atomic()?;
                    if atomics.len() <= atomic.geometry_index as usize {
                        atomics.resize(atomic.geometry_index as usize + 1, 0);
                    }
                    atomics[atomic.geometry_index as usize] = atomic.frame_index;
                }
                Some(RwSections::RwNodeName) => {
                    dummies.push(self.file.get_stream().read_string(header.section_size as usize)?);
                }
                Some(RwSections::RwAnim) => {
                    anim_nodes.push(self.read_anim_node()?);
                }
                _ => {
                    self.file.get_stream().skip(header.section_size as u64)?;
                }
            }
        }

        let model_type = if geometry_list.as_ref().map_or(false, |g| g.geometries.iter().any(|geo| geo.skin.is_some())) {
            DffModelType::Skin
        } else if dummies.iter().any(|d| d.to_lowercase().contains("wheel") || d.to_lowercase().contains("chassis")) {
            DffModelType::Vehicle
        } else {
            DffModelType::Generic
        };

        Ok(RwDff {
            model_type,
            version: version.unwrap_or_default(),
            version_number: version_number.unwrap_or_default(),
            geometry_list,
            frame_list,
            atomics,
            dummies,
            anim_nodes,
        })
    }

    fn read_frame_list(&mut self) -> Result<RwFrameList> {
        self.file.read_section_header()?; // Struct

        let frame_count = self.file.get_stream().read_u32()?;
        let mut frames = Vec::with_capacity(frame_count as usize);

        for _ in 0..frame_count {
            let rotation_matrix = RwMatrix3 {
                right: RwVector3 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                },
                up: RwVector3 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                },
                at: RwVector3 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                },
            };

            let coordinates_offset = RwVector3 {
                x: self.file.get_stream().read_f32()?,
                y: self.file.get_stream().read_f32()?,
                z: self.file.get_stream().read_f32()?,
            };

            let parent_frame = self.file.get_stream().read_i32()?;
            self.file.get_stream().skip(4)?; // Skip matrix creation internal flags

            frames.push(RwFrame {
                rotation_matrix,
                coordinates_offset,
                parent_frame,
            });
        }

        Ok(RwFrameList {
            frame_count,
            frames,
        })
    }

    fn read_atomic(&mut self) -> Result<RwAtomic> {
        self.file.read_section_header()?; // Struct

        let frame_index = self.file.get_stream().read_u32()?;
        let geometry_index = self.file.get_stream().read_u32()?;
        let flags = self.file.get_stream().read_u32()?;

        self.file.get_stream().skip(4)?; // Skip unused bytes

        Ok(RwAtomic {
            frame_index,
            geometry_index,
            flags,
        })
    }
    
    fn read_geometry_list(&mut self) -> Result<RwGeometryList> {
        let header = self.file.read_section_header()?; // Struct

        let geometric_object_count = self.file.get_stream().read_u32()?;
        let mut geometries = Vec::with_capacity(geometric_object_count as usize);

        for _ in 0..geometric_object_count {
            self.file.read_section_header()?; // Geometry
            self.file.read_section_header()?; // Struct
            let version_number = unpack_version(header.version_number);
            geometries.push(self.read_geometry(version_number)?);
        }

        Ok(RwGeometryList {
            geometric_object_count,
            geometries,
        })
    }

    fn read_geometry(&mut self, version_number: u32) -> Result<RwGeometry> {
        let flags = self.file.get_stream().read_u16()?;
        let texture_coordinates_count = self.file.get_stream().read_u8()?;
        let _native_geometry_flags = self.file.get_stream().read_u8()?;
        let triangle_count = self.file.get_stream().read_u32()?;
        let vertex_count = self.file.get_stream().read_u32()?;
        let _morph_target_count = self.file.get_stream().read_u32()?;

        if version_number < 0x34000 {
            self.file.get_stream().skip(12)?; // ambient, specular, diffuse
        }

        let is_textured_uv1 = (flags & (1 << 2)) != 0;
        let is_geometry_prelit = (flags & (1 << 3)) != 0;
        let is_textured_uv2 = (flags & (1 << 7)) != 0;

        let mut vertex_color_information = Vec::new();
        if is_geometry_prelit {
            for _ in 0..vertex_count {
                vertex_color_information.push(RwColor {
                    r: self.file.get_stream().read_u8()?,
                    g: self.file.get_stream().read_u8()?,
                    b: self.file.get_stream().read_u8()?,
                    a: self.file.get_stream().read_u8()?,
                });
            }
        }

        let mut texture_mapping_information = Vec::new();
        if is_textured_uv1 || is_textured_uv2 {
            for _ in 0..texture_coordinates_count {
                let mut tex_coords = Vec::new();
                for _ in 0..vertex_count {
                    tex_coords.push(RwTextureCoordinate {
                        u: self.file.get_stream().read_f32()?,
                        v: self.file.get_stream().read_f32()?,
                    });
                }
                texture_mapping_information.push(tex_coords);
            }
        }

        let mut triangle_information = Vec::new();
        for _ in 0..triangle_count {
            let vertex2 = self.file.get_stream().read_u16()?;
            let vertex1 = self.file.get_stream().read_u16()?;
            let material_id = self.file.get_stream().read_u16()?;
            let vertex3 = self.file.get_stream().read_u16()?;
            triangle_information.push(RwTriangle {
                vector: RwVector3 {
                    x: vertex1 as f32,
                    y: vertex2 as f32,
                    z: vertex3 as f32,
                },
                material_id,
            });
        }

        let bounding_sphere = Some(RwSphere {
            vector: RwVector3 {
                x: self.file.get_stream().read_f32()?,
                y: self.file.get_stream().read_f32()?,
                z: self.file.get_stream().read_f32()?,
            },
            radius: self.file.get_stream().read_f32()?,
        });

        let has_vertices = self.file.get_stream().read_u32()? != 0;
        let has_normals = self.file.get_stream().read_u32()? != 0;

        let mut vertex_information = Vec::new();
        if has_vertices {
            for _ in 0..vertex_count {
                vertex_information.push(RwVector3 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                });
            }
        }

        let mut normal_information = Vec::new();
        if has_normals {
            for _ in 0..vertex_count {
                normal_information.push(RwVector3 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                });
            }
        }

        let material_list = self.read_material_list()?;
        let section_size = self.file.read_section_header()?.section_size;
        let position = self.file.get_stream().get_position();
        let bin_mesh = self.read_bin_mesh()?;
        
        let mut skin = None;
        let next_header = self.file.read_section_header()?;
        if next_header.section_type == RwSections::RwSkin as u32 {
            skin = Some(self.read_skin(vertex_count)?);
        }

        self.file.get_stream().set_position(position + section_size as u64);

        Ok(RwGeometry {
            vertex_color_information,
            texture_coordinates_count,
            texture_mapping_information,
            has_vertices,
            has_normals,
            triangle_information,
            vertex_information,
            normal_information,
            bounding_sphere,
            material_list,
            bin_mesh,
            skin,
        })
    }

    fn read_material_list(&mut self) -> Result<RwMaterialList> {
        self.file.read_section_header()?; // Struct
        self.file.read_section_header()?; // MaterialList

        let material_instance_count = self.file.get_stream().read_u32()?;
        let mut material_indices = Vec::with_capacity(material_instance_count as usize);
        for _ in 0..material_instance_count {
            material_indices.push(self.file.get_stream().read_i32()?);
        }

        let mut material_data = Vec::with_capacity(material_instance_count as usize);
        for i in 0..material_instance_count {
            let material_index = material_indices[i as usize];
            if material_index == -1 {
                material_data.push(self.read_material()?);
            } else {
                material_data.push(material_data[material_index as usize].clone());
            }
        }

        Ok(RwMaterialList {
            material_instance_count,
            material_data,
        })
    }

    fn read_material(&mut self) -> Result<RwMaterial> {
        self.file.read_section_header()?; // Struct
        let header = self.file.read_section_header()?; // Material

        self.file.get_stream().skip(4)?; // Flags

        let color = RwColor {
            r: self.file.get_stream().read_u8()?,
            g: self.file.get_stream().read_u8()?,
            b: self.file.get_stream().read_u8()?,
            a: self.file.get_stream().read_u8()?,
        };

        self.file.get_stream().skip(4)?; // Unknown

        let is_textured = self.file.get_stream().read_u32()? > 0;

        let mut ambient = None;
        let mut specular = None;
        let mut diffuse = None;

        if header.version_number > 0x30400 {
            ambient = Some(self.file.get_stream().read_f32()?);
            specular = Some(self.file.get_stream().read_f32()?);
            diffuse = Some(self.file.get_stream().read_f32()?);
        }

        let mut texture = None;
        if is_textured {
            texture = Some(self.read_texture()?);
        }

        let size = self.file.read_section_header()?.section_size;
        self.file.get_stream().skip(size as u64)?;

        Ok(RwMaterial {
            color,
            is_textured,
            ambient,
            specular,
            diffuse,
            texture,
        })
    }

    fn read_texture(&mut self) -> Result<RwTexture> {
        self.file.read_section_header()?; // Struct
        self.file.read_section_header()?; // Texture

        let texture_data = self.file.get_stream().read_u32()?;
        let texture_filtering = (texture_data & 0xFF) as u8;
        let u_addressing = ((texture_data & 0xF00) >> 8) as u8;
        let v_addressing = ((texture_data & 0xF000) >> 12) as u8;
        let uses_mip_levels = (texture_data & (1 << 16)) != 0;

        let texture_name_size = self.file.read_section_header()?.section_size;
        let texture_name = self.file.get_stream().read_string(texture_name_size as usize)?;

        let size1 = self.file.read_section_header()?.section_size;
        self.file.get_stream().skip(size1 as u64)?;
        let size2 = self.file.read_section_header()?.section_size;
        self.file.get_stream().skip(size2 as u64)?;

        Ok(RwTexture {
            texture_filtering,
            u_addressing,
            v_addressing,
            uses_mip_levels,
            texture_name,
        })
    }

    fn read_bin_mesh(&mut self) -> Result<RwBinMesh> {
        self.file.read_section_header()?; // Struct

        self.file.get_stream().skip(4)?; // Flags
        let mesh_count = self.file.get_stream().read_u32()?;
        self.file.get_stream().skip(4)?; // Total number of indices

        let mut meshes = Vec::with_capacity(mesh_count as usize);
        for _ in 0..mesh_count {
            meshes.push(self.read_mesh()?);
        }

        Ok(RwBinMesh {
            mesh_count,
            meshes,
        })
    }

    fn read_mesh(&mut self) -> Result<RwMesh> {
        let index_count = self.file.get_stream().read_u32()?;
        let material_index = self.file.get_stream().read_u32()?;

        let mut indices = Vec::with_capacity(index_count as usize);
        for _ in 0..index_count {
            indices.push(self.file.get_stream().read_u32()?);
        }

        Ok(RwMesh {
            index_count,
            material_index,
            indices,
        })
    }

    fn read_skin(&mut self, vertex_count: u32) -> Result<RwSkin> {
        let bone_count = self.file.get_stream().read_u8()?;
        let used_bone_count = self.file.get_stream().read_u8()?;
        let max_weights_per_vertex = self.file.get_stream().read_u8()?;

        self.file.get_stream().skip(1)?; // Padding
        self.file.get_stream().skip(used_bone_count as u64)?; // Skipping special indices

        let mut bone_vertex_indices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            let mut indices = Vec::with_capacity(4);
            for _ in 0..4 {
                indices.push(self.file.get_stream().read_u8()?);
            }
            bone_vertex_indices.push(indices);
        }

        let mut vertex_weights = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            let mut weights = Vec::with_capacity(4);
            for _ in 0..4 {
                weights.push(self.file.get_stream().read_f32()?);
            }
            vertex_weights.push(weights);
        }

        let mut inverse_bone_matrices = Vec::with_capacity(bone_count as usize);
        for _ in 0..bone_count {
            inverse_bone_matrices.push(RwMatrix4 {
                right: RwVector4 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                    t: self.file.get_stream().read_f32()?,
                },
                up: RwVector4 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                    t: self.file.get_stream().read_f32()?,
                },
                at: RwVector4 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                    t: self.file.get_stream().read_f32()?,
                },
                transform: RwVector4 {
                    x: self.file.get_stream().read_f32()?,
                    y: self.file.get_stream().read_f32()?,
                    z: self.file.get_stream().read_f32()?,
                    t: self.file.get_stream().read_f32()?,
                },
            });
        }
        
        Ok(RwSkin {
            bone_count,
            used_bone_count,
            max_weights_per_vertex,
            bone_vertex_indices,
            vertex_weights,
            inverse_bone_matrices,
        })
    }

    fn read_anim_node(&mut self) -> Result<RwAnimNode> {
        self.file.get_stream().skip(4)?; // Skipping AnimVersion property (0x100)
        let bone_id = self.file.get_stream().read_i32()?;
        let bone_count = self.file.get_stream().read_i32()?;
        let mut bones = Vec::with_capacity(bone_count as usize);

        if bone_id == 0 {
            self.file.get_stream().skip(8)?; // Skipping flags and keyFrameSize properties
        }

        if bone_count > 0 {
            for _ in 0..bone_count {
                bones.push(RwBone {
                    bone_id: self.file.get_stream().read_i32()?,
                    bone_index: self.file.get_stream().read_i32()?,
                    flags: self.file.get_stream().read_i32()?,
                });
            }
        }

        Ok(RwAnimNode {
            bone_id,
            bones_count: bone_count,
            bones,
        })
    }
}