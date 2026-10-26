use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x0F)]
pub struct Goodbye {}