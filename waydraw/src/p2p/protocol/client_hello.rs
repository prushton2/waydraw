use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x01)]
pub struct ClientHello {
    pub version: (u8, u8, u8),
    pub window_width: u32,
    pub window_height: u32,
    pub supported_codecs: Vec<String>,
    
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    
}