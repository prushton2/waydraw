use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {}

impl super::IntoBytes for Heartbeat {
    fn into_bytes(self) -> Vec<u8> {
        vec![0x03]
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }
}