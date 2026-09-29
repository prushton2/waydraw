pub mod h264;

pub trait Encoder: Send + Sync {
    fn encode(&mut self, bytes: &Vec<u8>, window_size: (u32, u32)) -> Vec<u8>;
    fn decode(&mut self, bytes: &Vec<u8>) -> Vec<u8>;
}