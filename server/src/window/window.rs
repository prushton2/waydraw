use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use iced::Length::Fill;
use iced::Task;
use iced::alignment::Horizontal::Center;
use iced::widget::{space, button, column, container, row, text, text_input};

use p2p::protocol::{FromBytes, IntoBytes, ServerHello};

use crate::mouse;
use crate::screen_grabber::ScreenCapture;

use super::{Window, Message, ClientMessage, Monitor};

impl Window {
    pub fn boot() -> Self {
        let mut error = String::from("");

        let (monitors, monitor_labels) = match Self::read_monitors() {
            Ok(t) => t,
            Err(e) => {
                error = format!("Error reading monitors: {}\n\n", e);
                (vec![], vec![])
            }
        };
        
        let mut mouse: Box<dyn mouse::Mouse> = Box::new(mouse::DummyMouse::new());
        if !cfg!(debug_assertions) {
            mouse = Box::new(mouse::EnigoMouse::new());
        }

        let this = Self {
            p2p: Arc::new(RwLock::new(None)),
            mouse: mouse,
            h264_instance: Arc::new(Mutex::new(openh264::encoder::Encoder::new().unwrap())),

            client_window_size: Arc::new(std::sync::Mutex::new((100, 100))),
            connected: false,

            available_monitors: monitors,
            selected_monitor: None,
            recording: Arc::new(std::sync::RwLock::new(None)),

            pin: None,
            key: None,

            wait_reason: String::from(""),
            error: error,

            labels: monitor_labels,
        };

        this
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::Register => {
                if self.selected_monitor.is_none() {
                    self.error = "Error: Please select a monitor".to_owned();
                    return Task::none()
                }
                
                self.error = "".to_owned();
                self.wait_reason = "Registering...".to_owned();

                Task::perform(
                    async move {
                        let (server, key) = p2p::P2P::init().await.map_err(|e| e.to_string())?;

                        let pin = p2p::remote_key_store::generate_pin();
                        let key = key.to_string();
                        
                        match p2p::remote_key_store::set(&pin, &key.to_string()).await {
                            Ok(_) => {},
                            Err(e) => return Err(e)
                        };

                        Ok((Arc::new(RwLock::new(Some(server))), key, pin))
                    },
                    Message::AwaitClient
                )
            },

            Message::AwaitClient(result) => {
                let (p2p, key, pin) = match result {
                    Ok(t) => t,
                    Err(t) => {
                        self.error = t;
                        self.wait_reason = String::from("");
                        return Task::none()
                    }
                };

                self.p2p = p2p;
                self.key = Some(key);
                self.pin = Some(pin);

                let arc = self.p2p.clone();
                
                self.wait_reason = "Waiting for connection".to_owned();
                Task::perform(
                async move {
                    let mut lock = arc.write().await;
                    let p2p = lock.as_mut().unwrap();
                    match p2p.await_connection().await {
                        Ok(_) => {},
                        Err(e) => return Err(e.to_string())
                    };

                    Ok(())
                },
                    Message::SendHello,
                )
            },

            Message::SendHello(result) => {
                if let Err(e) = result {
                    self.error = format!("Error awaiting connection: {}", e);
                    self.wait_reason = String::from("");
                    return Task::none();
                }

                self.wait_reason = "Sending Hello".to_owned();
                let selected_monitor = &self.available_monitors[self.selected_monitor.unwrap_or(0)];

                // construct server info to send to client
                let p2p_arc = self.p2p.clone();

                let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>();

                let server_info = ServerHello {
                    version: (version[0], version[1], version[2]),
                    screen_width:  selected_monitor.resolution.0,
                    screen_height: selected_monitor.resolution.1
                };

                let server_info_bytes = server_info.into_bytes();

                Task::perform(
                    async move {
                        let p2p_lock = p2p_arc.read().await;
                        let p2p_ref = p2p_lock.as_ref().unwrap();
                        
                        let client_hello_bytes = p2p_ref.read().await.unwrap();
                        let client_hello_enum = match FromBytes::parse(&client_hello_bytes[..]) {
                            FromBytes::ClientHello(m) => m,
                            t => panic!("Expected client hello, received other bytes: {:?}", t)
                        };
                        
                        let _ = p2p_ref.send(&server_info_bytes).await;
                        
                        client_hello_enum
                    },
                    Message::Connect
                )
            },

            Message::Disconnect => {
                self.pin = None;
                self.key = None;
                self.connected = false;
                self.recording.read().unwrap().as_ref().unwrap().kill();
                *self.recording.write().unwrap() = None;
                self.error = String::from("");
                self.wait_reason = String::from("");
                let p2p_arc = self.p2p.clone();
                self.p2p = Arc::new(RwLock::new(None));

                Task::perform(
                async move {
                        let mut lock = p2p_arc.write().await;
                        if let Some(p2p) = lock.as_mut() {
                            let _ = p2p.close().await;
                        } else {
                            println!("Could not close connection");
                        }
                        
                        ()
                    },
                    Message::Null,
                )
            },

            Message::Connect(client_hello) => {
                let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

                if client_hello.version.0 != version[0] {
                    self.error = format!("Incompatible versions: Client {}.{}.{} and Server {}.{}.{}. Please update each app to the same major version.", client_hello.version.0, client_hello.version.1, client_hello.version.2, version[0], version[1], version[2]);
                    self.p2p = Arc::new(RwLock::new(None));
                    self.pin = None;
                    self.key = None;
                    return Task::none();
                }

                *self.client_window_size.lock().unwrap() = (client_hello.window_width, client_hello.window_height);
                self.connected = true;

                let monitor_id = self.available_monitors[self.selected_monitor.unwrap()].id.clone();

                let screencap = ScreenCapture::new(&monitor_id).unwrap();

                *self.recording.write().unwrap() = Some(screencap);

                Task::none()
            }

            Message::SelectMonitor(i) => {
                self.selected_monitor = Some(i);
                Task::none()
            },

            Message::None => {
                Task::none()
            },

            Message::Null(_) => {
                Task::none()
            },

            Message::ClientMessage(m) => {
                match m {
                    ClientMessage::MouseClick(button, state) => {
                        self.mouse.click_mouse(button, state);
                    },
                    ClientMessage::MouseMove(x, y) => {
                        let (offset_x, offset_y) = self.available_monitors[self.selected_monitor.unwrap()].position;
                        let scale = self.available_monitors[self.selected_monitor.unwrap_or(0)].scale;
                        let mouse_position = (
                            ((x as i32 + offset_x) as f32) / scale,
                            ((y as i32 + offset_y) as f32) / scale,
                        );
                        self.mouse.move_mouse(mouse_position.0 as i32, mouse_position.1 as i32);
                    },
                    ClientMessage::ClientWindowResize(x, y) => {
                        *self.client_window_size.lock().unwrap() = (x, y);
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.connected {
            return button("Disconnect").on_press(Message::Disconnect).into();
        }

        let pin = self.pin.clone().unwrap_or("".to_owned());
        let key = self.key.clone().unwrap_or("".to_owned());

        let mut buttons: Vec<iced::Element<'_, Message>> = vec![];

        for i in 0..self.labels.len() {
            let mut button = button(self.labels[i].as_str()).on_press(Message::SelectMonitor(i)).width(Fill);

            if Some(i) == self.selected_monitor {
                button = button.style(|theme, status| {
                    button::subtle(theme, status)
                });
            }

            buttons.push(
                button.into()
            );
            buttons.push(space().height(5).into())
        }

        container (
            column![
                text("Select a monitor").width(Fill).align_x(Center),
                iced::widget::Column::from_vec(buttons).width(Fill).align_x(Center),
                
                space().height(20),
                container(button("Allow Connections").on_press(Message::Register)).center_x(Fill),
                space().height(20),
                
                row![text("Pin"), space().width(24), text_input(&pin, &pin).on_input(|_| Message::None)],
                row![text("Key"), space().width(20), text_input(&key, &key).on_input(|_| Message::None)],
                
                text(&self.wait_reason).width(Fill).align_x(Center),
                text(&self.error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
            ]
            .max_width(400)

        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    fn read_monitors() -> Result<(Vec<Monitor>, Vec<String>), String> {
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
                    .find(|e| e.name == name)
                    .map(|e| e)
                    .unwrap();

                Monitor {
                    id: pinray_source.id.0.clone(),
                    name: pinray_source.name,
                    position: (displayinfo_source.x, displayinfo_source.y),
                    resolution: (pinray_source.width, pinray_source.height),
                    scale: displayinfo_source.scale_factor
                }
            })
            .collect::<Vec<Monitor>>();

        let monitor_labels = monitors
            .iter()
            .map(|e| 
                format!("{} ({}x{})", 
                    e.name,
                    e.resolution.0,
                    e.resolution.1
                )
            )
            .collect();

        return Ok((monitors, monitor_labels))
    }
}