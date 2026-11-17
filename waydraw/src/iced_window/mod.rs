use std::sync::Arc;

pub mod heartbeat;
pub mod screen_capture;
pub mod screens;
pub mod styles;
pub mod widgets;
pub mod window;

pub enum Screen {
    ClientConnected(screens::ClientConnected),
    Client(screens::Client),
    HostConnected(screens::HostConnected),
    Host(screens::Host),
    Settings(screens::Settings),
}
pub struct Window {
    pub config: crate::Config,
    pub screen: Screen,
    pub window_size: (usize, usize),
}

#[derive(Clone)]
pub enum Message {
    None,
    WindowResize(usize, usize),
    Tab(Arc<dyn Fn() -> Screen + Sync + Send>),
    /// Arguments are (String, String). First string is the hostname, second string is the key
    UpdateKnownHosts(String, String),
    /// Pass the key to remove the host
    RemoveKnownHost(String),

    HostMessage(screens::HostMessage),
    HostConnectedMessage(screens::host_connected::HostConnectedMessage),
    ClientMessage(screens::ClientMessage),
    ClientConnectedMessage(screens::client_connected::ClientConnectedMessage),
    SettingsMessage(screens::SettingsMessage),
}
