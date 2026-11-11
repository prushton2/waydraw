pub mod screens;
pub mod window;
pub mod styles;
pub mod widgets;

#[derive(Clone)]
pub enum Screen {
    ClientConnected(screens::ClientConnected),
    Client(screens::Client),
    HostConnected(screens::HostConnected),
    Host(screens::Host),
    Settings(screens::Settings)
}

pub struct Window {
    pub config: crate::Config,
    pub screen: Screen,
}

#[derive(Clone)]
pub enum Message {
    None,
    Tab(Screen),
    HostMessage(screens::HostMessage),
    HostConnectedMessage(screens::host_connected::HostConnectedMessage),
    ClientMessage(screens::ClientMessage),
    ClientConnectedMessage(screens::client_connected::ClientConnectedMessage),
    SettingsMessage(screens::SettingsMessage),
}
