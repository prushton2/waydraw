use std::sync::Arc;

use iced_core::image::{Allocation, Error};
use tokio::sync::{Mutex, RwLock};

use crate::p2p::protocol::{ClientHello, ServerHello};
use crate::{encoding::Encoder, window::modules::receive_stream};

use crate::p2p::protocol::mouse_click::{MouseButton, MouseState};

pub mod window;
mod modules;

pub struct Window {
    config: crate::Config,
    p2p: Arc<RwLock<Option<crate::p2p::P2P>>>,
    encoder: Arc<Mutex<Option<Box<dyn Encoder>>>>,
    ui_state: modules::ui_state::UIState,

    // Client side stuff
    allocation: Option<iced_core::image::Allocation>,
    server_info: Option<ServerHello>,
    
    // Host side stuff
    client_info: Option<ClientHello>,


    // Misc
    window_size: (usize, usize)
}

#[derive(Clone)]
pub enum Message {
    UIUpdate(modules::ui_state::UIUpdate),
    P2PMessage(receive_stream::P2PMessage),
    RemoveKnownHost(String),

    // Client side stuff
    ClientConnectFlow(modules::client::ConnectFlow),
    MouseMove(i32, i32),
    MouseClick(MouseButton, MouseState),
    // ImageAllocated(Result<Allocation, Error>),
    
    // Host side stuff
    HostConnectFlow(modules::host::ConnectFlow),
    Disconnect,

    // Misc
    WindowResize(usize, usize),
    None,
    Empty(())
}