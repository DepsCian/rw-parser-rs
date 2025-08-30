//! # TXD Parser
//!
//! A parser for RenderWare TXD (Texture Dictionary) files, used for storing
//! textures in Grand Theft Auto 3, Vice City, and San Andreas.
//!
//! ## Features
//!
//! - Parses texture metadata, including name, dimensions, and format.
//! - Supports decompression of DXT1, DXT3, and DXT5 (BC1, BC2, BC3) textures.
//! - Extracts mipmap levels for supported formats.
//! - Deserializes texture data into a structured `RwTxd` format.
//!
//! ## Example
//!
//! ```no_run
//! use rw_parser_rs::renderware::txd::txd_parser::TxdParser;
//! use std::fs;
//!
//! let file_data = fs::read("path/to/your/textures.txd").unwrap();
//! let mut parser = TxdParser::new(&file_data);
//! let txd_data = parser.parse().unwrap();
//!
//! println!("Texture count: {}", txd_data.texture_dictionary.texture_count);
//! ```

use crate::renderware::rw_file::RwFile;
use crate::renderware::utils::image_format_enums::{PaletteType, PlatformType};
use std::io::Result;
use texpresso;

use serde::Serialize;

/// Represents the top-level structure of a parsed TXD file.
///
/// This struct contains the texture dictionary, which holds all the
/// individual textures in the file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwTxd {
    /// The texture dictionary containing all texture data.
    pub texture_dictionary: RwTextureDictionary,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwTextureDictionary {
    pub texture_count: u16,
    pub texture_natives: Vec<RwTextureNative>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RwTextureNative {
    pub platform_id: u32,
    pub filter_mode: u8,
    pub u_addressing: u8,
    pub v_addressing: u8,
    pub texture_name: String,
    pub mask_name: String,
    pub raster_format: u32,
    pub d3d_format: String,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub mipmap_count: u8,
    pub raster_type: u8,
    pub alpha: bool,
    pub cube_texture: bool,
    pub auto_mip_maps: bool,
    pub compressed: bool,
    pub mipmaps: Vec<Vec<u8>>,
}

/// The main parser for TXD files.
///
/// This struct holds the file buffer and provides the `parse` method to
/// deserialize the TXD texture data.
pub struct TxdParser<'a> {
    file: RwFile<'a>,
}

impl<'a> TxdParser<'a> {
    /// Creates a new `TxdParser` instance with the given file buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A byte slice containing the raw TXD file data.
    pub fn new(buffer: &'a [u8]) -> Self {
        TxdParser {
            file: RwFile::new(buffer),
        }
    }

    /// Parses the entire TXD file buffer.
    ///
    // This method reads the root `TextureDictionary` section and all the
    // `TextureNative` sections within it.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `RwTxd` data or an `std::io::Error`
    /// if a parsing error occurs.
    pub fn parse(&mut self) -> Result<RwTxd> {
        Ok(RwTxd {
            texture_dictionary: self.read_texture_dictionary()?,
        })
    }

    fn read_texture_dictionary(&mut self) -> Result<RwTextureDictionary> {
        self.file.read_section_header()?; // Struct
        self.file.read_section_header()?; // TextureDictionary

        let texture_count = self.file.get_stream().read_u16()?;
        self.file.get_stream().skip(2)?;

        let mut texture_natives = Vec::with_capacity(texture_count as usize);
        for _ in 0..texture_count {
            texture_natives.push(self.read_texture_native()?);
        }
        
        let size = self.file.read_section_header()?.section_size;
        self.file.get_stream().skip(size as u64)?;

        Ok(RwTextureDictionary {
            texture_count,
            texture_natives,
        })
    }

    fn read_texture_native(&mut self) -> Result<RwTextureNative> {
        self.file.read_section_header()?; // Struct
        self.file.read_section_header()?; // TextureNative

        let platform_id = self.file.get_stream().read_u32()?;
        let flags = self.file.get_stream().read_u32()?;

        let filter_mode = (flags & 0xFF) as u8;
        let u_addressing = ((flags & 0xF00) >> 8) as u8;
        let v_addressing = ((flags & 0xF000) >> 12) as u8;

        let texture_name = self.file.get_stream().read_string(32)?;
        let mask_name = self.file.get_stream().read_string(32)?;

        let raster_format = self.file.get_stream().read_u32()?;
        let d3d_format = self.file.get_stream().read_string(4)?;
        let width = self.file.get_stream().read_u16()?;
        let height = self.file.get_stream().read_u16()?;
        let depth = self.file.get_stream().read_u8()?;
        let mipmap_count = self.file.get_stream().read_u8()?;
        let raster_type = self.file.get_stream().read_u8()?;
        let compression_flags = self.file.get_stream().read_u8()?;

        let alpha = (compression_flags & (1 << 0)) != 0;
        let cube_texture = (compression_flags & (1 << 1)) != 0;
        let auto_mip_maps = (compression_flags & (1 << 2)) != 0;
        let compressed = (compression_flags & (1 << 3)) != 0;

        let palette_type = (raster_format >> 13) & 0b11;

        let mut mipmaps = Vec::new();
        let palette = if palette_type != PaletteType::PaletteNone as u32 {
            self.read_palette(palette_type, depth)?
        } else {
            Vec::new()
        };

        for i in 0..mipmap_count {
            let raster_size = self.file.get_stream().read_u32()?;
            let raster = self.file.get_stream().read(raster_size as usize)?;
            
            if i == 0 {
                 let bitmap = if !palette.is_empty() {
                    // Palette decoding is not implemented yet
                    vec![]
                } else if platform_id == PlatformType::D3d8 as u32 && compression_flags != 0 {
                    self.get_bitmap_with_dxt(&format!("DXT{}", compression_flags), &raster, width, height)?
                } else if platform_id == PlatformType::D3d9 as u32 && compressed {
                    self.get_bitmap_with_dxt(&d3d_format, &raster, width, height)?
                } else {
                    // Raw RGBA decoding is not implemented yet
                    vec![]
                };
                mipmaps.push(bitmap);
            }
        }
        
        let size = self.file.read_section_header()?.section_size;
        self.file.get_stream().skip(size as u64)?;

        Ok(RwTextureNative {
            platform_id,
            filter_mode,
            u_addressing,
            v_addressing,
            texture_name,
            mask_name,
            raster_format,
            d3d_format,
            width,
            height,
            depth,
            mipmap_count,
            raster_type,
            alpha,
            cube_texture,
            auto_mip_maps,
            compressed,
            mipmaps,
        })
    }

    fn read_palette(&mut self, palette_type: u32, depth: u8) -> Result<Vec<u8>> {
        let size = if palette_type == PaletteType::Palette8 as u32 { 1024 } else if depth == 4 { 64 } else { 128 };
        self.file.get_stream().read(size)
    }

    fn get_bitmap_with_dxt(&self, dxt_type: &str, raster: &[u8], width: u16, height: u16) -> Result<Vec<u8>> {
        let format = match dxt_type {
            "DXT1" => texpresso::Format::Bc1,
            "DXT2" => texpresso::Format::Bc2,
            "DXT3" => texpresso::Format::Bc2,
            "DXT4" => texpresso::Format::Bc3,
            "DXT5" => texpresso::Format::Bc3,
            _ => return Ok(Vec::new())
        };

        let mut decoded = vec![0; width as usize * height as usize * 4];
        format.decompress(raster, width as usize, height as usize, &mut decoded);
        
        Ok(decoded)
    }
}