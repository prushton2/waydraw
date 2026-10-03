use std::sync::Arc;

use tokio::sync::RwLock;

use iced_core::image::{Allocation, Error};

use crate::p2p::protocol::{ClientHello, ServerHello};
use crate::screen_capture::ScreenCapture;
use crate::window::modules::heartbeat;
use crate::window::window::Monitor;
use crate::{encoding::Codec, window::modules::receive_stream};

use crate::p2p::protocol::mouse_click::{MouseButton, MouseState};

pub mod window;
mod modules;

pub struct Window {
    pub config: crate::Config,
    heartbeat: Arc<std::sync::Mutex<heartbeat::Heartbeat>>,

    p2p: Arc<RwLock<Option<crate::p2p::P2P>>>,
    encoder: Arc<std::sync::Mutex<Option<Box<dyn Codec>>>>,
    ui_state: modules::ui_state::UIState,
    monitors: Vec<Monitor>,
    selected_monitor: String,

    // Client side stuff
    allocation: Option<iced_core::image::Allocation>,
    server_info: Option<ServerHello>,
    
    // Host side stuff
    client_info: Option<ClientHello>,
    mouse: Box<dyn crate::mouse::Mouse>,
    video_recorder: Arc<std::sync::RwLock<Option<ScreenCapture>>>,

    // Misc
    window_size: (usize, usize)
}

#[derive(Clone)]
pub enum Message {
    UIUpdate(modules::ui_state::UIUpdate),
    P2PMessage(receive_stream::P2PMessage),
    RemoveKnownHost(String),
    HeartbeatMessage(heartbeat::HeartbeatMessage),

    // Client side stuff
    ImageAllocated(Result<Allocation, Error>),
    ClientConnectFlow(modules::client::ConnectFlow),
    MouseMove(i32, i32),
    MouseClick(MouseButton, MouseState),
    // ImageAllocated(Result<Allocation, Error>),
    
    // Host side stuff
    HostConnectFlow(modules::host::ConnectFlow),
    SelectMonitor(String),
    Disconnect,

    // Misc
    WindowResize(usize, usize),
    ChangeTheme(iced::Theme),
    None,
    Empty(())
}