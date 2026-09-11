use std::sync::Arc;

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

    handle: Option<iced_core::image::Handle>,

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