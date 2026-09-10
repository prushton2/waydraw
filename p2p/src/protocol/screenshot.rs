#[derive(Debug, Clone)]
pub struct Screenshot {
    pub pixels: Vec<(u8, u8, u8)>
}

impl super::IntoBytes for Screenshot {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![0x20];

        for i in self.pixels {
            bytes.push(i.0);
            bytes.push(i.1);
            bytes.push(i.2);
        }

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut this = Self {
            pixels: vec![]
        };

        for i in 0..bytes.len() {
            if i%3 == 0 {
                this.pixels.push(
                    (bytes[i], bytes[i+1], bytes[i+2])
                )
            }
        }

        this
    }
}