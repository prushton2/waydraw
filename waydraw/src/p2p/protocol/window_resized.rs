use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x02)]
pub struct WindowResized {
    pub window_width: u32,
    pub window_height: u32
}