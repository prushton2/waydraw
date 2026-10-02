use std::sync::{Arc, Mutex};
use iced::Task;
use tokio::sync::RwLock;

use iced::Subscription;

use crate::p2p::protocol::{self, IntoBytes};
use crate::window::modules;
use crate::window::modules::screencap_stream::ScreencapStreamParameters;
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

        let monitors = match read_monitors() {
            Ok(t) => t,
            Err(_) => vec![]
        };
        
        Self {
            config: crate::config::Config::load_or_generate(),
            p2p: Arc::new(RwLock::new(None)),
            encoder: Arc::new(Mutex::new(None)),
            ui_state: ui_state::UIState::Host { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") },
            
            monitors: monitors,
            selected_monitor: String::from(""),

            allocation: None,
            server_info: None,

            client_info: None,
            mouse: mouse,
            video_recorder: Arc::new(std::sync::RwLock::new(None)),

            window_size: (256, 256),
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
                receive_stream::update(self, message) 
            },

            Message::ImageAllocated(result) => {
                if let Ok(allocation) = result {
                    self.allocation = Some(allocation);
                }

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
                // let window_size = self.window_size.clone();
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
            Message::SelectMonitor(v) => {
                self.selected_monitor = v;
                Task::none()
            }

            // Misc
            Message::ChangeTheme(theme) => {
                self.config.theme = theme;
                self.config.write();
                Task::none()
            }
            Message::WindowResize(x, y) => {
                self.window_size = (x, y);
                if let UIState::ConnectedClient = self.ui_state {
                    let p2p_arc = self.p2p.clone();
                    return Task::perform(async move {
                            let lock = p2p_arc.read().await;
                            if let Some(p2p) = lock.as_ref() {
                                let message = protocol::WindowResized {
                                    window_width: x as u32,
                                    window_height: y as u32
                                };
                                let _ = p2p.send(&message.into_bytes()).await;
                            }
                            ()
                        },
                        Message::Empty
                    )
                }
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

    if let UIState::ConnectedHost = window.ui_state {
        let screencap_stream = ScreencapStreamParameters {
            recording: window.video_recorder.clone(),
            encoder: window.encoder.clone(),
            p2p: window.p2p.clone(),
        };

        subscriptions.push(
            iced::Subscription::run_with(screencap_stream, screencap_stream::screencap_stream),
        );
    }

    return iced::Subscription::batch(subscriptions)
}

pub struct Monitor {
    pub id: String,
    pub position: (i32, i32),
    pub resolution: (u32, u32),
    pub scale: f32,
    pub label: String,
}

fn read_monitors() -> Result<Vec<Monitor>, String> {
    let positions = display_info::DisplayInfo::all().unwrap_or_default();

    let monitors = pinray::enumerate_sources()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|e| {
            match e {
                pinray::CaptureSource::Display(display) => Some(display),
                _ => None
            }
        })
        .map(|pinray_source| {
            // pinray ids are `display:<name>`, where <name> is the same device name display_info reports (`DP-1`, `\\.\DISPLAY1`, ...).
            let name = pinray_source.id.0.strip_prefix("display:").unwrap_or(&pinray_source.id.0);

            let displayinfo_source = positions
                .iter()
                .find(|e| e.name == name || e.id.to_string() == name)
                .map(|e| e)
                .unwrap();

            Monitor {
                id: pinray_source.id.0.clone(),
                position: (displayinfo_source.x, displayinfo_source.y),
                resolution: (pinray_source.width, pinray_source.height),
                label: format!("{} {}x{}", pinray_source.name, pinray_source.width, pinray_source.height),
                #[cfg(target_os = "macos")] // Macos reports points, not pixels, which are already scaled, so we set the scale to 1.0
                scale: 1.0,
                #[cfg(not(target_os = "macos"))]
                scale: displayinfo_source.scale_factor,
            }
        })
        .collect::<Vec<Monitor>>();

    return Ok(monitors)
}