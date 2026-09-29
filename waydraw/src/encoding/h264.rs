use openh264::{decoder::Decoder, encoder::Encoder, formats, nal_units};
use openh264::formats::YUVSource;

pub struct H264 {
    encoder: Encoder,
    decoder: Decoder
}

impl H264 {
    pub fn new() -> Result<Self, openh264::Error> {
        let encoder = Encoder::new()?;
        let decoder = Decoder::new()?;

        Ok(Self {
            encoder,
            decoder
        })
    }
}

impl super::Encoder for H264 {
    fn encode(&mut self, bytes: &Vec<u8>, window_size: (u32, u32)) -> Vec<u8> {
        let rgba_slice = formats::RgbaSliceU8::new(bytes, (window_size.0 as usize, window_size.1 as usize));
        let yuv_buffer = formats::YUVBuffer::from_rgba8_source(rgba_slice);

        let encoded = match self.encoder.encode(&yuv_buffer) {
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

    fn decode(&mut self, bytes: &Vec<u8>) -> Vec<u8> {
        let mut rgba8: Vec<u8> = vec![];

        for packet in nal_units(bytes) {
            if let Ok(Some(yuv)) = self.decoder.decode(packet) {   
                rgba8 = vec![0; yuv.rgba8_len()];
                yuv.write_rgba8(&mut rgba8);
            }
        };

        rgba8
    }
}