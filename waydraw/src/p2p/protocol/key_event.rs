use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x12)]
pub struct KeyEvent {

}
