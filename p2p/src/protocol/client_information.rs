#[derive(Debug, Copy, Clone)]
pub struct ClientInformation {
    pub width: u32,
    pub height: u32
}

impl super::IntoBytes for ClientInformation {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x01];
        buf.extend_from_slice(&self.width.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let w_bytes: [u8; 4] = (&bytes[0..4]).try_into().unwrap();
        let h_bytes: [u8; 4] = (&bytes[4..8]).try_into().unwrap();

        Self {
            width:  u32::from_be_bytes(w_bytes),
            height: u32::from_be_bytes(h_bytes)
        }
    }
}