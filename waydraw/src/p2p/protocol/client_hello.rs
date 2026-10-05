use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    pub version: (u8, u8, u8),
    pub window_width: u32,
    pub window_height: u32,
    pub supported_codecs: Vec<String>
}

impl super::IntoBytes for ClientHello {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x01];
        let bytes = rmp_serde::to_vec(&self).unwrap();
        buf.extend_from_slice(&bytes);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
}