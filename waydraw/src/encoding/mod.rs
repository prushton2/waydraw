pub mod h264;
pub use h264::H264;

pub trait Codec: Send + Sync {
    fn encode(&mut self, bytes: &Vec<u8>, window_size: (u32, u32)) -> Vec<u8>;
    fn decode(&mut self, bytes: &Vec<u8>) -> Vec<u8>;
}

struct CodecEntry {
    pub check_fn: fn() -> Result<(), String>,
    pub new_fn: fn() -> Result<Box<dyn Codec>, String>
}

const CODECS: [(&str, CodecEntry); 1] = [
    ("H.264", CodecEntry {
        check_fn: H264::check,
        new_fn: || {
            match H264::new() {
                Ok(t) => Ok(Box::new(t)),
                Err(e) => Err(e.to_string())
            }
        }
    })
];

// A codec must be able to encode and decode on the client and host device. Crates that can only encode/decode must
// be paired together so one module can do both.
pub fn get_compatible_codecs() -> Vec<&'static str> {
    let mut valid_codecs: Vec<&str> = vec![];
    for (codec, entry) in &CODECS {
        match (entry.check_fn)() {
            Ok(_) => valid_codecs.push(codec),
            Err(_) => {}
        }
    }

    valid_codecs
}

pub fn get_codec(codec: &str) -> Result<Box<dyn Codec>, String> {
    let mut codec_ref: Option<&CodecEntry> = None;

    for (codec_name, entry) in &CODECS {
        if *codec_name == codec {
            codec_ref = Some(entry)
        }
    };

    let codec_ref = match codec_ref {
        Some(t) => t,
        None => return Err("Codec not found".to_string())
    };

    (codec_ref.new_fn)()
}