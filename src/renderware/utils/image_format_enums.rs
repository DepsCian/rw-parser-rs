#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteType {
    PaletteNone = 0,
    Palette4 = 1,
    Palette8 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    D3d8 = 8,
    D3d9 = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterFormat {
    Raster1555 = 0x01,
    Raster565 = 0x02,
    Raster4444 = 0x03,
    RasterLum = 0x04,
    Raster8888 = 0x05,
    Raster888 = 0x06,
    Raster555 = 0x0A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum D3dFormat {
    D3dfmtA8l8,
    D3dDxt1,
    D3dDxt2,
    D3dDxt3,
    D3dDxt4,
    D3dDxt5,
}