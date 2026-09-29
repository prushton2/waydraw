use std::sync::Arc;
use iced::Task;
use tokio::sync::{RwLock, Mutex};

use iced::Subscription;

use crate::p2p::protocol::{self, IntoBytes};
use crate::window::modules;
use crate::window::modules::ui_state::UIState;
use crate::window::modules::*;

use super::Window;
use super::Message;

impl Window {
    pub fn boot() -> Self {
        let mut mouse: Box<dyn crate::mouse::Mouse> = Box::new(crate::mouse::DummyMouse::new());
        if !cfg!(debug_assertions) {
            mouse = Box::new(crate::mouse::EnigoMouse::new());
        }
        
        Self {
            config: crate::Config::load_or_generate(),
            p2p: Arc::new(RwLock::new(None)),
            encoder: Arc::new(Mutex::new(None)),
            ui_state: ui_state::UIState::Host { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") },

            allocation: None,
            server_info: None,

            client_info: None,
            mouse: mouse,

            window_size: (256, 256)
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UIUpdate(message) => {
                ui_state::update(self, message);
                Task::none()
            },
            Message::RemoveKnownHost(host) => {
                self.config.known_hosts.remove(&host);
                self.config.write();
                Task::none()
            },
            Message::P2PMessage(message) => {
                let _ = receive_stream::update(self, message);
                Task::none()
            },

            // Client stuff
            Message::ClientConnectFlow(message) => {
                modules::client::update(self, message).map(Message::ClientConnectFlow)
            },
            Message::MouseClick(button, state) => {
                let p2p_arc = self.p2p.clone();
                Task::perform(async move {
                        let p2p_lock = p2p_arc.read().await;
                        if let Some(p2p) = p2p_lock.as_ref() {
                            
                            let message = protocol::MouseClick {
                                button, state
                            };

                            let _ = p2p.send(&message.into_bytes()).await;
                        }

                        ()
                    },
                    Message::Empty
                )
            },
            Message::MouseMove(x, y) => {
                let p2p_arc = self.p2p.clone();
                Task::perform(async move {
                        let p2p_lock = p2p_arc.read().await;
                        if let Some(p2p) = p2p_lock.as_ref() {
                            
                            let message = protocol::MouseMove {
                                x: x as u32, y: y as u32
                            };

                            let _ = p2p.send(&message.into_bytes()).await;
                        }

                        ()
                    },
                    Message::Empty
                )
            },

            // Host stuff
            Message::HostConnectFlow(message) => {
                modules::host::update(self, message).map(Message::HostConnectFlow)
            },
            Message::Disconnect => {
                modules::host::update(self, modules::host::ConnectFlow::Disconnect).map(Message::HostConnectFlow)
            },

            Message::WindowResize(x, y) => {
                self.window_size = (x, y);
                Task::none()
            }
            Message::None => {
                Task::none()
            },
            Message::Empty(_) => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        return ui_state::view(self);
    }
}

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![
        iced::window::resize_events().map(|(_id, size)| Message::WindowResize(size.width as usize, size.height as usize))
    ];

    if let UIState::ConnectedClient | UIState::ConnectedHost = window.ui_state {
        subscriptions.push(
            iced::Subscription::run_with(receive_stream::P2PObject(window.p2p.clone()), receive_stream::p2p_stream),
        );
    }

    return iced::Subscription::batch(subscriptions)
}