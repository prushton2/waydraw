use std::sync::Arc;

use tokio::sync::Mutex;

use crate::encoding::Encoder;

pub mod window;
mod modules;

pub struct Window {
    config: crate::Config,
    p2p: Arc<Mutex<Option<crate::p2p::P2P>>>,
    encoder: Arc<Mutex<Option<Box<dyn Encoder>>>>,
    ui_state: modules::ui_state::UIState
}

#[derive(Clone)]
pub enum Message {
    UIUpdate(modules::ui_state::UIUpdate),
    RemoveKnownHost(String),
    None
}