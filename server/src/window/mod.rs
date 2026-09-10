use std::sync::Arc;
use p2p::{p2p::P2PError, protocol::{ClientHello, mouse_click::{MouseButton, MouseState}}};
use tokio::sync::RwLock;
use winit::monitor::MonitorHandle;
use xcap::image::RgbaImage;

use crate::mouse;

pub mod subscriptions;
pub mod window;
pub mod downscale;

pub struct Window {
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
    client_window_size: (u32, u32),
    mouse: Box<dyn mouse::Mouse>,
    connected: bool,
    
    available_monitors: Vec<MonitorHandle>,
    selected_monitor: Option<usize>,

    pin: Option<String>,
    key: Option<String>,
    
    wait_reason: String,
    error: String,

    // AAAA
    labels: Vec<String>
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
    ScreenshotCaptured(RgbaImage),
    None,
    Null(())
}

#[derive(Clone)]
pub enum ClientMessage {
    MouseClick(MouseButton, MouseState),
    MouseMove(u32, u32),
    ClientWindowResize(u32, u32)
}