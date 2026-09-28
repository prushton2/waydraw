#[derive(Debug, Clone)]
pub struct Screenshot {
    pub bytes: Vec<u8>
}

impl super::IntoBytes for Screenshot {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![0x20];
        bytes.extend(self.bytes);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes)
        }
    }
}