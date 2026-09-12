#[derive(Debug, Clone)]
pub struct CompressedScreenshot {
    pub bytes: Vec<u8>
}

impl super::IntoBytes for CompressedScreenshot {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![0x21];
        bytes.extend_from_slice(&self.bytes);

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let this = Self {
            bytes: bytes.iter().map(|i| *i).collect()
        };

        this
    }
}