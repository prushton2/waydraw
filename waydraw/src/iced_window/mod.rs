use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;

mod screens;
pub mod window;
pub mod styles;

pub type TokioOpt<T> = Arc<RwLock<Option<T>>>;
pub type SyncOpt<T> = Arc<Mutex<Option<T>>>;

#[derive(Clone)]
pub enum Screen {
    ClientConnected(screens::ClientConnected),
    Client(screens::Client),
    HostConnected(screens::HostConnected),
    Host(screens::Host),
    Settings(screens::Settings)
}

pub struct Window {
    screen: Screen
}

#[derive(Clone)]
pub enum Message {
    None,
    Tab(Screen),
    HostMessage(screens::HostMessage),
    ClientMessage(screens::ClientMessage),
    SettingsMessage(screens::SettingsMessage),
}
