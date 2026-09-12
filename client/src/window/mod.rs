use std::sync::Arc;

use iced_core::image::{Allocation, Error};
use p2p::p2p::P2PError;
use tokio::sync::RwLock;

use p2p::protocol::{Screenshot, CompressedScreenshot, ServerHello};
use p2p::protocol::mouse_click::{MouseButton, MouseState};

pub mod subscriptions;
pub mod window;

#[derive(Default)]
pub struct Window {
    server_info: Option<ServerHello>,
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
    connected: bool,

    allocation: Option<iced_core::image::Allocation>,

    known_size: (usize, usize),
    
    pin_textbox: String,
    key_textbox: String,

    wait_reason: String,
    error: String,
}

#[derive(Clone)]
pub enum Message {
    MouseMove(f32, f32),
    MouseClick(MouseButton, MouseState),
    WindowResized((usize, usize)),

    ScreenshotReceived(ScreenshotType),
    ImageAllocated(Result<Allocation, Error>),

    PinSubmitted,
    P2PCreated(Result<(Arc<RwLock<Option<p2p::P2P>>>, ServerHello), P2PError>),
    
    PINTextbox(String),
    KeyTextbox(String),
    Sent(Result<(), String>),

    // None,
    Null(())
}

#[derive(Clone)]
pub enum ScreenshotType {
    Uncompressed(Screenshot),
    Compressed(CompressedScreenshot)
}