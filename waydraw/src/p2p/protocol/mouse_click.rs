use crate::input::{MouseButton, MouseState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x11)]
pub struct MouseClick {
    pub button: MouseButton,
    pub state: MouseState,
}