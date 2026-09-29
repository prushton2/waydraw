use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

use fast_image_resize::ResizeOptions;
use fast_image_resize::Resizer;
use fast_image_resize::images::Image;
use fast_image_resize::images::ImageRef;

use crate::encoding::Encoder;
use crate::screen_capture::ScreenCapture;
use crate::window::Message;
use crate::p2p::protocol::IntoBytes;

// struct ScreencapStreamParameters(Arc<InnerScreencapStreamParameters>);
#[derive(Clone)]
pub struct ScreencapStreamParameters {
    pub recording: Arc<std::sync::RwLock<Option<ScreenCapture>>>,
    pub encoder: Arc<std::sync::Mutex<Option<Box<dyn Encoder>>>>,
    pub p2p: Arc<RwLock<Option<crate::p2p::P2P>>>,
    pub client_window_size:(u32, u32),
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

        let client_window_size = parameters.client_window_size;

        if client_window_size.0 == 0 || client_window_size.1 == 0 {
            return Some((Message::Empty(()), parameters))
        }

        let image_bytes = image.to_tight_bytes().unwrap();

        let opts = ResizeOptions::new()
            .resize_alg(fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear))
            .use_alpha(false);

        let src = ImageRef::new(image.width, image.height, &image_bytes, fast_image_resize::PixelType::U8x4).unwrap();
        let mut dst = Image::new(client_window_size.0, client_window_size.1, fast_image_resize::PixelType::U8x4);
        let _ = Resizer::new().resize(&src, &mut dst, Some(&opts));

        let dst_bytes = &dst.into_vec();

        let mut encoded_bytes: Vec<u8> = vec![];

        if let Some(encoder) = parameters.encoder.lock().unwrap().as_mut() {
            encoded_bytes = encoder.encode(dst_bytes, client_window_size);
        }

        let p2p_lock = parameters.p2p.read().await;
        let p2p_ref = p2p_lock.as_ref().unwrap();

        let frame = crate::p2p::protocol::Screenshot {
            bytes: encoded_bytes
        };

        let _ = p2p_ref.send(&frame.into_bytes()).await;

        drop(p2p_lock);

        Some((Message::Empty(()), parameters))
    })
}