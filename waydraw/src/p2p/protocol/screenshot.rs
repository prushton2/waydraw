use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x20)]
pub struct Screenshot {
    pub bytes: Vec<u8>,
    pub width: usize,
    pub height: usize
}