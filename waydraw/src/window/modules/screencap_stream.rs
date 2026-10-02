use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::RwLock;

use crate::encoding::Codec;
use crate::screen_capture::ScreenCapture;
use crate::window::Message;
use crate::p2p::protocol::IntoBytes;

// struct ScreencapStreamParameters(Arc<InnerScreencapStreamParameters>);
#[derive(Clone)]
pub struct ScreencapStreamParameters {
    pub recording: Arc<std::sync::RwLock<Option<ScreenCapture>>>,
    pub encoder: Arc<Mutex<Option<Box<dyn Codec>>>>,
    pub p2p: Arc<RwLock<Option<crate::p2p::P2P>>>,
}

impl Hash for ScreencapStreamParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // `self.0` is a fresh Arc allocated on every `subscription()` call (i.e. after every
        // message), so its own pointer changes every time and would make iced tear down and
        // respawn this stream constantly, tearing frames mid-write on the p2p connection and
        // permanently desyncing its length-prefixed framing (client hangs in read_exact -> freeze).
        // Hash on h264_instance instead: it's created once in `Window::boot` and never replaced,
        // so the identity stays stable for the life of the app and the stream is kept running.
        Arc::as_ptr(&self.encoder).hash(state);
    }
}

pub fn screencap_stream(params: &ScreencapStreamParameters) -> impl iced::futures::Stream<Item = Message> + use<> {
    let params_clone = params.clone();
   
    iced::futures::stream::unfold(params_clone, |parameters| async move {
        tokio::time::sleep(std::time::Duration::from_millis(1000/30)).await;

        let image_result = {
            let recording_lock = parameters.recording.read().unwrap();
            recording_lock.as_ref().unwrap().latest()
        };

        let image = match image_result {
            Some(t) => t,
            None => {
                drop(image_result);
                // println!("No image found");
                return Some((Message::Empty(()), parameters))
            }
        };

        let image_bytes = image.to_tight_bytes().unwrap();

        let mut encoded_bytes: Vec<u8> = vec![];

        if let Some(encoder) = parameters.encoder.lock().unwrap().as_mut() {
            encoded_bytes = encoder.encode(&image_bytes, (image.width, image.height));
        }

        let p2p_lock = parameters.p2p.read().await;
        let p2p_ref = p2p_lock.as_ref().unwrap();

        let frame = crate::p2p::protocol::Screenshot {
            bytes: encoded_bytes,
            width: image.width as usize,
            height: image.height as usize
        };

        let _ = p2p_ref.send(&frame.into_bytes()).await;

        drop(p2p_lock);

        Some((Message::Empty(()), parameters))
    })
}