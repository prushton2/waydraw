#[derive(Debug, Copy, Clone)]
pub struct ClientHello {
    pub version: (u8, u8, u8),
    pub window_width: u32,
    pub window_height: u32
}

impl super::IntoBytes for ClientHello {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x01];
        buf.extend_from_slice(&self.window_width.to_be_bytes());
        buf.extend_from_slice(&self.window_height.to_be_bytes());
        buf.push(self.version.0);
        buf.push(self.version.1);
        buf.push(self.version.2);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let w_bytes: [u8; 4] = (&bytes[0..4]).try_into().unwrap();
        let h_bytes: [u8; 4] = (&bytes[4..8]).try_into().unwrap();

        Self {
            version: (bytes[8], bytes[9], bytes[10]),
            window_width:  u32::from_be_bytes(w_bytes),
            window_height: u32::from_be_bytes(h_bytes)
        }
    }
}