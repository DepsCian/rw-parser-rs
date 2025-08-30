use num_derive::FromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum RwSections {
    RwStruct = 0x0001,
    RwString = 0x0002,
    RwExtension = 0x0003,
    RwTexture = 0x0006,
    RwMaterial = 0x0007,
    RwMaterialList = 0x0008,
    RwFrameList = 0x000E,
    RwGeometry = 0x000F,
    RwClump = 0x0010,
    RwAtomic = 0x0014,
    RwTextureNative = 0x0015,
    RwTextureDictionary = 0x0016,
    RwGeometryList = 0x001A,
    RwSkin = 0x116,
    RwAnim = 0x11E,

    RwMaterialEffectsPLG = 0x0120,

    RwReflectionMaterial = 0x0253F2FC,
    RwNodeName = 0x0253F2FE,
}