use openh264::{encoder::Encoder, formats};

pub struct H264 {
    h264_instance: Encoder
}

impl H264 {
    pub fn new() -> Result<Self, openh264::Error> {
        let encoder = Encoder::new()?;

        Ok(Self {
            h264_instance: encoder
        })
    }
}

impl super::Encoder for H264 {
    fn encode(&mut self, bytes: &Vec<u8>, window_size: (u32, u32)) -> Vec<u8> {
        let rgba_slice = formats::RgbaSliceU8::new(bytes, (window_size.0 as usize, window_size.1 as usize));
        let yuv_buffer = formats::YUVBuffer::from_rgba8_source(rgba_slice);

        let encoded = match self.h264_instance.encode(&yuv_buffer) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("h264 encode failed for {}x{}: {e}", window_size.0, window_size.1);
                return vec![];
            }
        };
        let mut bytes = vec![];
        encoded.write_vec(&mut bytes);

        return bytes;
    }
}