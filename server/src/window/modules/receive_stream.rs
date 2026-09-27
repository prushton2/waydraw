use std::{hash::Hash, sync::Arc};

use iced::Task;
use p2p::protocol::{FromBytes, mouse_click::{MouseButton, MouseState}};
use tokio::sync::RwLock;

use crate::window::{Message, Window};

#[derive(Clone)]
pub enum P2PMessage {
    MouseClick(MouseButton, MouseState),
    MouseMove(u32, u32),
    ClientWindowResize(u32, u32)
}

pub struct P2PObject(pub Arc<RwLock<Option<p2p::P2P>>>);

impl Hash for P2PObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // This object requires a write lock on the p2p read object, which only one can exist at a time.
        // As a result, there can only be one p2p_stream, so we return a constant hash.
        "P2PObject".hash(state);
    }
}

pub fn p2p_stream(feed: &P2PObject) -> impl iced::futures::Stream<Item = Message> + use<> {
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
                return Some((Message::P2PMessage(P2PMessage::MouseClick(t.button, t.state)), p2p))
            },
            FromBytes::MouseMove(t) => {
                return Some((Message::P2PMessage(P2PMessage::MouseMove(t.x, t.y)), p2p))
            },
            FromBytes::WindowResized(t) => {
                return Some((Message::P2PMessage(P2PMessage::ClientWindowResize(t.window_width, t.window_height)), p2p))
            },
            FromBytes::UnknownInstruction(_) => {},
            _ => {}
        }

        Some((Message::Null(()), p2p))
    })
}

pub fn update(this: &mut Window, message: P2PMessage) -> Task<P2PMessage> {
    match message {
        P2PMessage::MouseClick(button, state) => {
            this.mouse.click_mouse(button, state);
            Task::none()
        },
        P2PMessage::MouseMove(x, y) => {
            let (offset_x, offset_y) = this.available_monitors[this.selected_monitor.unwrap()].position;
            let scale = this.available_monitors[this.selected_monitor.unwrap_or(0)].scale;
            let mouse_position = (
                ((x as i32 + offset_x) as f32) / scale,
                ((y as i32 + offset_y) as f32) / scale,
            );
            this.mouse.move_mouse(mouse_position.0 as i32, mouse_position.1 as i32);
            Task::none()
        },
        P2PMessage::ClientWindowResize(x, y) => {
            *this.client_window_size.lock().unwrap() = (x, y);
            Task::none()
        }
    }
}