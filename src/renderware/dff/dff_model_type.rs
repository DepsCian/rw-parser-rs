use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DffModelType {
    Generic,
    Skin,
    Vehicle,
}