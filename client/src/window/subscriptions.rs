use std::sync::Arc;

use tokio::sync::RwLock;

use iced::{Subscription, window};

use p2p::protocol::FromBytes;

use super::{Message, ScreenshotType, Window};

struct P2PObject(Arc<RwLock<Option<p2p::P2P>>>);

impl std::hash::Hash for P2PObject {
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
            FromBytes::Screenshot(t) => {
                return Some((Message::ScreenshotReceived(ScreenshotType::Uncompressed(t)), p2p))
            },
            FromBytes::CompressedScreenshot(t) => {
                return Some((Message::ScreenshotReceived(ScreenshotType::Compressed(t)), p2p))
            }
            _ => {}
        }

        Some((Message::Null(()), p2p))
    })
}

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![
        window::resize_events().map(|(_id, size)| Message::WindowResized((size.width as usize, size.height as usize)))
    ];

    if window.connected {
        subscriptions.push(iced::Subscription::run_with(P2PObject(window.p2p.clone()), p2p_stream));
    }

    return iced::Subscription::batch(subscriptions);
}