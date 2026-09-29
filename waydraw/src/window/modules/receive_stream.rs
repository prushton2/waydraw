use std::{hash::Hash, sync::Arc};
use tokio::sync::RwLock;

use fast_image_resize::{ResizeOptions, Resizer, images::{Image, ImageRef}};
use iced::{Task, widget::image};
use crate::{p2p::protocol::{self, FromBytes, mouse_click::{MouseButton, MouseState}}, window::modules::ui_state::UIState};
use crate::p2p;

use crate::window::{Message, Window};

#[derive(Clone)]
pub enum P2PMessage {
    MouseClick(MouseButton, MouseState),
    MouseMove(u32, u32),
    WindowResize(u32, u32),
    ScreenshotReceived(protocol::Screenshot)
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
                return Some((Message::None, p2p))
            }
        };

        let response = match p2p_ref.read().await {
            Ok(t) => t,
            Err(_) => {
                drop(p2p_lock);
                return Some((Message::None, p2p))
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
                return Some((Message::P2PMessage(P2PMessage::WindowResize(t.window_width, t.window_height)), p2p))
            },
            FromBytes::Screenshot(t) => {
                return Some((Message::P2PMessage(P2PMessage::ScreenshotReceived(t)), p2p))
            }
            FromBytes::UnknownInstruction(_) => {},
            _ => {}
        }

        Some((Message::None, p2p))
    })
}

pub fn update(this: &mut Window, message: P2PMessage) -> Task<Message> {
    match message {
        P2PMessage::MouseClick(button, state) => {
            this.mouse.click_mouse(button, state);
            Task::none()
        },
        P2PMessage::MouseMove(x, y) => {
            let selected_monitor = this.monitors.iter().filter(|e| e.id == this.selected_monitor).nth(0).unwrap();

            let (offset_x, offset_y) = selected_monitor.position;
            let client_window_res = (this.client_info.unwrap().window_width, this.client_info.unwrap().window_height);
            let scale = selected_monitor.scale;

            let mouse_pct = (
                x as f32 / client_window_res.0 as f32,
                y as f32 / client_window_res.1 as f32
            );

            let mouse_scaled_to_target_res = (
                mouse_pct.0 * selected_monitor.resolution.0 as f32,
                mouse_pct.1 * selected_monitor.resolution.1 as f32,
            );

            let mouse_position = (
                ((mouse_scaled_to_target_res.0 as i32 + offset_x) as f32) / scale,
                ((mouse_scaled_to_target_res.1 as i32 + offset_y) as f32) / scale,
            );
            
            this.mouse.move_mouse(mouse_position.0 as i32, mouse_position.1 as i32);
            Task::none()
        },
        P2PMessage::WindowResize(x, y) => {
            if let Some(window_size) = this.client_info.as_mut() {
                window_size.window_width = x;
                window_size.window_height = y;
            }

            Task::none()
        },
        P2PMessage::ScreenshotReceived(screenshot) => {
            if let UIState::ConnectedClient = this.ui_state {
                let mut encoder_lock = this.encoder.lock().unwrap();

                let image_pixels = if let Some(encoder) = encoder_lock.as_mut() {
                    encoder.decode(&screenshot.bytes)
                } else {
                    return Task::none()
                };

                if image_pixels.len() != screenshot.width * screenshot.height * 4 {
                    // Decoder produced no frame (e.g. waiting for a keyframe) or size mismatch
                    return Task::none();
                }

                let opts = ResizeOptions::new()
                    .resize_alg(fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear))
                    .use_alpha(false);

                let src = ImageRef::new(screenshot.width as u32, screenshot.height as u32, &image_pixels, fast_image_resize::PixelType::U8x4).unwrap();
                let mut dst = Image::new(this.window_size.0 as u32, this.window_size.1 as u32, fast_image_resize::PixelType::U8x4);
                let _ = Resizer::new().resize(&src, &mut dst, Some(&opts));

                let dst_bytes = dst.into_vec();

                let handle = image::Handle::from_rgba(this.window_size.0 as u32, this.window_size.1 as u32, dst_bytes);

                return image::allocate(handle).map(Message::ImageAllocated)
            }
            Task::none()
        }
    }
}