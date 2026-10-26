use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x10)]
pub struct MouseMove {
    pub x: u32,
    pub y: u32
}