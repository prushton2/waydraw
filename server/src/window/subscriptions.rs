use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

use iced::Subscription;

use p2p::protocol::FromBytes;
use p2p::protocol::IntoBytes;

use super::{Window, Message, ClientMessage};
use crate::downscale;
use crate::screen_grabber::ScreenCapture;

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![];

    
    if window.connected {
        subscriptions.push(
            iced::Subscription::run_with(P2PObject(window.p2p.clone()), p2p_stream),
        );
    }
    
    
    if window.connected && window.recording.is_some() {

        subscriptions.push(
            iced::Subscription::run_with((ScreenGrabberObject(window.recording.clone()), window.client_window_size, P2PObject(window.p2p.clone())), screencap_stream)
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

struct ScreenGrabberObject(Arc<Option<ScreenCapture>>);

impl Hash for ScreenGrabberObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

fn screencap_stream((
    recording, 
    client_window_size, 
    p2pobject): &(ScreenGrabberObject, (u32, u32), P2PObject)
) -> impl iced::futures::Stream<Item = Message> + use<> {
    
    let recording = recording.0.clone();
    let cws = client_window_size.clone();
    let p2p_clone = p2pobject.0.clone();

    iced::futures::stream::unfold((recording, cws, p2p_clone), |(recording, client_window_size, p2p)| async move {
        tokio::time::sleep(std::time::Duration::from_millis(1000/10)).await;
        
        let mut start = std::time::Instant::now();
        let recording_ref = recording.as_ref().as_ref().unwrap();

        println!("Choose monitor: {:?}", start.elapsed());
        start = std::time::Instant::now();

        let image_result = recording_ref.latest();

        println!("Capture Image: {:?}", start.elapsed());
        start = std::time::Instant::now();

        
        let image = match image_result {
            Some(t) => t,
            None => {
                drop(image_result);
                println!("No image found");
                return Some((Message::Null(()), (recording, client_window_size, p2p)))
            }
        };
        
        let downscaled_bytes = downscale::downscale_frame(image.to_tight_bytes().unwrap(), (image.width, image.height), client_window_size);

        println!("Downscale: {:?}", start.elapsed());
        start = std::time::Instant::now();

        let compressed = downscaled_bytes;

        println!("Compress: {:?}", start.elapsed());
        start = std::time::Instant::now();

        let p2p_lock = p2p.read().await;
        let p2p_ref = p2p_lock.as_ref().unwrap();

        let frame = p2p::protocol::CompressedScreenshot {
            bytes: compressed
        };

        let _ = p2p_ref.send(&frame.into_bytes()).await;
        println!("Send: {:?}", start.elapsed());

        drop(p2p_lock);
        
        Some((Message::Null(()), (recording, client_window_size, p2p)))
    })
}