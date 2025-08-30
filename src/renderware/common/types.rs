use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwVector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub t: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwMatrix3 {
    pub right: RwVector3,
    pub up: RwVector3,
    pub at: RwVector3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwMatrix4 {
    pub right: RwVector4,
    pub up: RwVector4,
    pub at: RwVector4,
    pub transform: RwVector4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RwColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwTextureCoordinate {
    pub u: f32,
    pub v: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwTriangle {
    pub vector: RwVector3,
    pub material_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RwSphere {
    pub vector: RwVector3,
    pub radius: f32,
}