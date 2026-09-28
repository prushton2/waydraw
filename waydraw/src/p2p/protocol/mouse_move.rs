#[derive(Debug, Copy, Clone)]
pub struct MouseMove {
    pub x: u32,
    pub y: u32
}

impl super::IntoBytes for MouseMove {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x10];
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let w_bytes: [u8; 4] = (&bytes[0..4]).try_into().unwrap();
        let h_bytes: [u8; 4] = (&bytes[4..8]).try_into().unwrap();

        Self {
            x:  u32::from_be_bytes(w_bytes),
            y: u32::from_be_bytes(h_bytes)
        }
    }
}