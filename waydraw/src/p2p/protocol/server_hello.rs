use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x00)]
pub struct ServerHello {
    pub name: String,
    pub version: (u8, u8, u8),
    pub screen_width: u32,
    pub screen_height: u32,
    pub selected_codec: String
}