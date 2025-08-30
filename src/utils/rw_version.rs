use std::collections::HashMap;

pub struct RwVersion {
    versions: HashMap<u32, String>,
}

impl RwVersion {
    pub fn new() -> Self {
        let mut versions = HashMap::new();
        versions.insert(0x31000, "RenderWare 3.1.0.0 (III on PS2)".to_string());
        versions.insert(0x32000, "RenderWare 3.2.0.0 (III on PC)".to_string());
        versions.insert(0x33002, "RenderWare 3.3.0.2 (III on PC, VC on PS2)".to_string());
        versions.insert(0x34003, "RenderWare 3.4.0.3 (VC on PC)".to_string());
        versions.insert(0x34005, "RenderWare 3.4.0.5 (III on PS2, VC on Android/PC)".to_string());
        versions.insert(0x35000, "RenderWare 3.5.0.0 (III/VC on Xbox)".to_string());
        versions.insert(0x36003, "RenderWare 3.6.0.3 (SA)".to_string());
        RwVersion { versions }
    }

    pub fn get_version_string(&self, version_number: u32) -> Option<String> {
        self.versions.get(&version_number).cloned()
    }
}

pub fn unpack_version(version: u32) -> u32 {
    if version & 0xFFFF0000 != 0 {
        (version >> 14 & 0x3FF00) + 0x30000 | (version >> 16 & 0x3F)
    } else {
        version
    }
}

pub fn unpack_build(version: u32) -> u32 {
    if version & 0xFFFF0000 != 0 {
        version & 0xFFFF
    } else {
        0
    }
}