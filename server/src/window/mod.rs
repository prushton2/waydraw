use std::sync::Arc;
use p2p::{p2p::P2PError, protocol::{ClientHello, mouse_click::{MouseButton, MouseState}}};
use pinray::DisplaySource;
use tokio::sync::{Mutex, RwLock};

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
    mouse: Box<dyn mouse::Mouse>,
    h264_instance: Arc<Mutex<openh264::encoder::Encoder>>,

    // Shared with the long-lived screencap subscription (see subscriptions.rs) so it can
    // observe resizes/reconnects in place without the subscription's identity changing.
    client_window_size: Arc<std::sync::Mutex<(u32, u32)>>,
    connected: bool,

    available_monitors: Vec<Monitor>,
    selected_monitor: Option<usize>,
    recording: Arc<std::sync::RwLock<Option<ScreenCapture>>>,

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