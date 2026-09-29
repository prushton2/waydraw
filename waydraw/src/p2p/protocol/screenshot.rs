use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]

#[derive(Serialize, Deserialize)]
pub struct Screenshot {
    pub bytes: Vec<u8>,
    pub width: usize,
    pub height: usize
}

impl super::IntoBytes for Screenshot {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x20];
        buf.extend_from_slice(&rmp_serde::to_vec(&self).unwrap()[..]);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let message: Self = rmp_serde::from_slice(&bytes).unwrap();
        message
    }
}