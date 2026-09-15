use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::RwLock;

use fast_image_resize::ResizeOptions;
use fast_image_resize::Resizer;
use fast_image_resize::images::Image;
use fast_image_resize::images::ImageRef;

use openh264::formats;
use openh264::encoder;

use iced::Subscription;

use p2p::protocol::FromBytes;
use p2p::protocol::IntoBytes;

use super::{Window, Message, ClientMessage};
use crate::screen_grabber::ScreenCapture;

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![];

    if window.connected {
        subscriptions.push(
            iced::Subscription::run_with(P2PObject(window.p2p.clone()), p2p_stream),
        );
    }
    
    if window.connected && window.recording.read().unwrap().is_some() {
        let params = Arc::new(InnerScreencapStreamParameters {
            recording: window.recording.clone(),
            h264_instance: window.h264_instance.clone(),
            p2p: window.p2p.clone(),
            client_window_size: window.client_window_size.clone()
        });


        subscriptions.push(
            iced::Subscription::run_with(ScreencapStreamParameters(params.clone()), screencap_stream)
        );
    };

    return iced::Subscription::batch(subscriptions);
}

// Receiver
struct P2PObject(Arc<RwLock<Option<p2p::P2P>>>);

impl Hash for P2PObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

fn p2p_stream(feed: &P2PObject) -> impl iced::futures::Stream<Item = Message> + use<> {
    let p2p = feed.0.clone();

    iced::futures::stream::unfold(p2p, |p2p| async move {
        let p2p_lock = p2p.read().await;
        
        let p2p_ref = match p2p_lock.as_ref() {
            Some(t) => t,
            None => {
                drop(p2p_lock);
                return Some((Message::Null(()), p2p))
            }
        };

        let response = match p2p_ref.read().await {
            Ok(t) => t,
            Err(_) => {
                drop(p2p_lock);
                return Some((Message::Null(()), p2p))
            }
        };

        drop(p2p_lock);
        match FromBytes::parse(&response[..]) {
            FromBytes::MouseClick(t) => {
                return Some((Message::ClientMessage(ClientMessage::MouseClick(t.button, t.state)), p2p))
            },
            FromBytes::MouseMove(t) => {
                return Some((Message::ClientMessage(ClientMessage::MouseMove(t.x, t.y)), p2p))
            },
            FromBytes::WindowResized(t) => {
                return Some((Message::ClientMessage(ClientMessage::ClientWindowResize(t.window_width, t.window_height)), p2p))
            },
            FromBytes::UnknownInstruction(_) => {},
            _ => {}
        }

        Some((Message::Null(()), p2p))
    })
}

struct ScreencapStreamParameters(Arc<InnerScreencapStreamParameters>);
struct InnerScreencapStreamParameters {
    recording: Arc<std::sync::RwLock<Option<ScreenCapture>>>,
    h264_instance: Arc<Mutex<encoder::Encoder>>,
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
    client_window_size: Arc<std::sync::Mutex<(u32, u32)>>,
}

impl Hash for ScreencapStreamParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // `self.0` is a fresh Arc allocated on every `subscription()` call (i.e. after every
        // message), so its own pointer changes every time and would make iced tear down and
        // respawn this stream constantly, tearing frames mid-write on the p2p connection and
        // permanently desyncing its length-prefixed framing (client hangs in read_exact -> freeze).
        // Hash on h264_instance instead: it's created once in `Window::boot` and never replaced,
        // so the identity stays stable for the life of the app and the stream is kept running.
        Arc::as_ptr(&self.0.h264_instance).hash(state);
    }
}

fn screencap_stream(params: &ScreencapStreamParameters) -> impl iced::futures::Stream<Item = Message> + use<> {

    let params_clone = params.0.clone();

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
                println!("No image found");
                return Some((Message::Null(()), parameters))
            }
        };

        let client_window_size = *parameters.client_window_size.lock().unwrap();

        if client_window_size.0 == 0 || client_window_size.1 == 0 {
            println!("Client window size is 0");
            return Some((Message::Null(()), parameters))
        }

        let bytes = image.to_tight_bytes().unwrap();

        let opts = ResizeOptions::new()
            .resize_alg(fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear))
            .use_alpha(false);

        let src = ImageRef::new(image.width, image.height, &bytes, fast_image_resize::PixelType::U8x4).unwrap();
        let mut dst = Image::new(client_window_size.0, client_window_size.1, fast_image_resize::PixelType::U8x4);
        let _ = Resizer::new().resize(&src, &mut dst, Some(&opts));

        let dst_bytes = &dst.into_vec();
        let rgba_slice = formats::RgbaSliceU8::new(dst_bytes, (client_window_size.0 as usize, client_window_size.1 as usize));
        let yuv_buffer = formats::YUVBuffer::from_rgba8_source(rgba_slice);

        let bytes = {
            let mut h264_lock = parameters.h264_instance.lock().await;
            // encode() errors on e.g. an odd/unsupported resolution. This stream is now
            // long-lived (see the Hash impl above) and nothing respawns it if its future
            // panics, so an unhandled error here would silently end the video feed for the
            // rest of the session - just drop the frame and keep going instead.
            let encoded = match h264_lock.encode(&yuv_buffer) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("h264 encode failed for {}x{}: {e}", client_window_size.0, client_window_size.1);
                    drop(h264_lock);
                    return Some((Message::Null(()), parameters))
                }
            };
            let mut bytes = vec![];
            encoded.write_vec(&mut bytes);
            bytes
            // h264_lock and encoded (not Send, holds raw pointers into the encoder)
            // are dropped here, before the next .await
        };

        let p2p_lock = parameters.p2p.read().await;
        let p2p_ref = p2p_lock.as_ref().unwrap();

        let frame = p2p::protocol::H264Packet {
            bytes: bytes
        };

        let _ = p2p_ref.send(&frame.into_bytes()).await;

        drop(p2p_lock);

        Some((Message::Null(()), parameters))
    })
}