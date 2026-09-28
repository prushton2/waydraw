use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::{encoding::Encoder, mouse, screen_capture::ScreenCapture, window::modules::receive_stream::P2PMessage};

pub mod subscriptions;
pub mod window;
pub mod modules;

// pinray and display_info reliably provide different props, so i manually merge them together
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub position: (i32, i32),
    pub resolution: (u32, u32),
    pub scale: f32,
}

pub struct Window {
    p2p: Arc<RwLock<Option<p2p::P2P>>>,
    mouse: Box<dyn mouse::Mouse>,
    encoder: Arc<Mutex<Option<Box<dyn Encoder>>>>,

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
    ConnectFlow(modules::connect_flow::ConnectFlow),
    SelectMonitor(usize),
    P2PMessage(P2PMessage),
    None,
    Null(())
}

