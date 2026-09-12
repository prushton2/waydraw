use std::sync::Arc;
use p2p::{p2p::P2PError, protocol::{ClientHello, mouse_click::{MouseButton, MouseState}}};
use pinray::DisplaySource;
use tokio::sync::RwLock;

use crate::{mouse, screen_grabber::ScreenCapture};

pub mod subscriptions;
pub mod window;

// The position isnt given in pinray, so i get both and tie them together
pub struct Monitor {
    pub source: DisplaySource,
    pub position: (i32, i32),
}

pub struct Window {
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
    client_window_size: (u32, u32),
    mouse: Box<dyn mouse::Mouse>,
    connected: bool,
    
    available_monitors: Vec<Monitor>,
    selected_monitor: Option<usize>,
    recording: Arc<Option<ScreenCapture>>,

    pin: Option<String>,
    key: Option<String>,
    
    wait_reason: String,
    error: String,

    // AAAA
    labels: Vec<String>,
}

#[derive(Clone)]
pub enum Message {
    Register,
    AwaitClient(Result<(Arc<RwLock<Option<p2p::P2P>>>, String, String), Arc<P2PError>>),
    SendHello(Result<(), String>),
    SelectMonitor(usize),
    ClientMessage(ClientMessage),
    Disconnect,
    Connect(ClientHello),
    None,
    Null(())
}

#[derive(Clone)]
pub enum ClientMessage {
    MouseClick(MouseButton, MouseState),
    MouseMove(u32, u32),
    ClientWindowResize(u32, u32)
}