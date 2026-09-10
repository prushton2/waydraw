use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

use xcap;
use xcap::Monitor;

use iced::Subscription;

use super::{Window, Message, ClientMessage};

use p2p::protocol::FromBytes;

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![];

    
    if window.connected {
        subscriptions.push(
            iced::Subscription::run_with(P2PObject(window.p2p.clone()), p2p_stream),
        );
    }
    
    
    if window.connected && let Some(selected_monitor_index) = window.selected_monitor {
        let selected_monitor_name = window.available_monitors[selected_monitor_index].name();

        subscriptions.push(
            iced::Subscription::run_with(selected_monitor_name.unwrap(), screencap_stream)
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

fn screencap_stream(selected_monitor_name: &String) -> impl iced::futures::Stream<Item = Message> + use<> {
    
    let selected_monitor_name = selected_monitor_name.clone();

    iced::futures::stream::unfold(selected_monitor_name, |selected_monitor_name| async move {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let monitors = Monitor::all().unwrap();
        let mut selected_monitor: &Monitor = &monitors[0];
        
        
        for current_monitor in &monitors {
            if current_monitor.name().unwrap() == selected_monitor_name {
                selected_monitor = current_monitor;
                break;
            }
        }

        let image_result = selected_monitor.capture_image();

        let image = match image_result {
            Ok(t) => t,
            Err(t) => {
                return Some((Message::Null(()), selected_monitor_name))
            }
        };

        Some((Message::ScreenshotCaptured(image), selected_monitor_name))
    })
}