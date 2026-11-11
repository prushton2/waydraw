pub mod screen_capture;
pub mod screens;
pub mod styles;
pub mod widgets;
pub mod window;

#[derive(Clone)]
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
    Tab(Screen),
    HostMessage(screens::HostMessage),
    HostConnectedMessage(screens::host_connected::HostConnectedMessage),
    ClientMessage(screens::ClientMessage),
    ClientConnectedMessage(screens::client_connected::ClientConnectedMessage),
    SettingsMessage(screens::SettingsMessage),
}
