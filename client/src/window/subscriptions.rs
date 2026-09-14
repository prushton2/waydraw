use std::{hash::Hash, sync::Arc};

use openh264::{decoder, formats::YUVSource, nal_units};
use tokio::sync::{Mutex, RwLock};

use iced::{Subscription, window};

use p2p::protocol::{FromBytes, IntoBytes};

use super::{Message, ScreenshotType, Window};

struct P2PStreamParameters(Arc<InnerP2PStreamParameters>);
struct InnerP2PStreamParameters {
    h264_instance: Arc<Mutex<decoder::Decoder>>,
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
}

impl Hash for P2PStreamParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

fn p2p_stream(feed: &P2PStreamParameters) -> impl iced::futures::Stream<Item = Message> + use<> {
    let p2p_clone = feed.0.clone();

    iced::futures::stream::unfold(p2p_clone, |parameters| async move {
        let p2p_lock = parameters.p2p.read().await;
        
        let p2p_ref = match p2p_lock.as_ref() {
            Some(t) => t,
            None => {
                drop(p2p_lock);
                return Some((Message::Null(()), parameters))
            }
        };

        let response = match p2p_ref.read().await {
            Ok(t) => t,
            Err(_) => {
                drop(p2p_lock);
                return Some((Message::Null(()), parameters))
            }
        };

        drop(p2p_lock);
        match FromBytes::parse(&response[..]) {
            FromBytes::Screenshot(t) => {
                return Some((Message::ScreenshotReceived(ScreenshotType::Uncompressed(t)), parameters))
            },
            FromBytes::CompressedScreenshot(t) => {
                return Some((Message::ScreenshotReceived(ScreenshotType::Compressed(t)), parameters))
            },
            FromBytes::H264Packet(h264_bytes) => {
                let mut h264_lock = parameters.h264_instance.lock().await;
                let mut rgba8: Vec<u8> = vec![];

                for packet in nal_units(&h264_bytes.into_bytes()) {
                    if let Ok(Some(yuv)) = h264_lock.decode(packet) {
                        rgba8 = vec![0; yuv.rgba8_len()];
                        yuv.write_rgba8(&mut rgba8);
                    }
                }
                
            }
            _ => {}
        }

        Some((Message::Null(()), parameters))
    })
}

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![
        window::resize_events().map(|(_id, size)| Message::WindowResized((size.width as usize, size.height as usize)))
    ];

    if window.connected {
        let parameters = Arc::new(InnerP2PStreamParameters {
            h264_instance: window.h264_instance.clone(),
            p2p: window.p2p.clone()
        });
        subscriptions.push(iced::Subscription::run_with(P2PStreamParameters(parameters), p2p_stream));
    }

    return iced::Subscription::batch(subscriptions);
}