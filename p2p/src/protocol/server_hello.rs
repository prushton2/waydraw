use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    pub name: String,
    pub version: (u8, u8, u8),
    pub screen_width: u32,
    pub screen_height: u32
}

impl super::IntoBytes for ServerHello {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x00];
        buf.extend_from_slice(&rmp_serde::to_vec(&self).unwrap()[..]);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let message: Self = rmp_serde::from_slice(&bytes).unwrap();
        message
    }
}