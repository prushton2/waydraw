use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::Mutex;

use iced::Length::Fill;
use iced::{Subscription, Task};
use iced::alignment::Horizontal::Center;
use iced::widget::{space, button, column, container, row, text, text_input};

use p2p::p2p::P2PError;
use p2p::protocol::mouse_click::{MouseButton, MouseState};
use p2p::protocol::{FromBytes, IntoBytes, ServerHello};

use winit::monitor::MonitorHandle;

use crate::mouse;

pub struct Window {
    p2p: Arc<Mutex<Option<p2p::P2P>>>,
    mouse: Box<dyn mouse::Mouse>,
    connected: bool,
    
    available_monitors: Vec<MonitorHandle>,
    selected_monitor: Option<usize>,

    pin: Option<String>,
    key: Option<String>,
    
    wait_reason: String,
    error: String,

    // AAAA
    labels: Vec<String>
}

#[derive(Clone)]
pub enum Message {
    Register,
    AwaitClient(Result<(Arc<Mutex<Option<p2p::P2P>>>, String, String), Arc<P2PError>>),
    SendHello(Result<(), String>),
    SelectMonitor(usize),
    ClientMessage(ClientMessage),
    Disconnect,
    Connect(()),
    None,
    Null(())
}

#[derive(Clone)]
pub enum ClientMessage {
    MouseClick(MouseButton, MouseState),
    MouseMove(u32, u32)
}

impl Window {
    pub fn boot(monitors: Vec<MonitorHandle>) -> Self {
        let monitor_labels = monitors
            .iter()
            .map(|e| 
                format!("{} ({}x{}@{}hz)", 
                    e.name().unwrap_or("".to_owned()), 
                    e.size().width, 
                    e.size().height, 
                    e.refresh_rate_millihertz().unwrap_or(0) / 1000
                )
            ).collect();

        let mut mouse: Box<dyn mouse::Mouse> = Box::new(mouse::DummyMouse::new());
        if !cfg!(debug_assertions) {
            mouse = Box::new(mouse::EnigoMouse::new());
        }

        let this = Self {
            p2p: Arc::new(Mutex::new(None)),
            mouse: mouse,
            connected: false,

            available_monitors: monitors,
            selected_monitor: None,

            pin: None,
            key: None,

            wait_reason: String::from(""),
            error: String::from(""),

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
                        let (server, key) = p2p::P2P::init().await.map_err(|e| Arc::new(e))?;

                        let pin = p2p::remote_key_store::generate_key();
                        let key = key.to_string();
                        p2p::remote_key_store::set(&pin, &key.to_string()).await;

                        Ok((Arc::new(Mutex::new(Some(server))), key, pin))
                    },
                    Message::AwaitClient
                )
            },

            Message::AwaitClient(result) => {
                let (p2p, key, pin) = match result {
                    Ok(t) => t,
                    Err(t) => {
                        self.error = String::from((*t).clone());
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
                    let mut lock = arc.lock().await;
                    let p2p = lock.as_mut().unwrap();
                    let _ = p2p.await_connection().await;

                    Ok(())
                },
                    Message::SendHello,
                )
            },

            Message::SendHello(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                self.wait_reason = "Sending Hello".to_owned();
                let selected_monitor = &self.available_monitors[self.selected_monitor.unwrap()];

                // construct server info to send to client
                let p2p_arc = self.p2p.clone();

                let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

                let server_info_bytes = ServerHello {
                    version: (version[0], version[1], version[2]),
                    screen_width:  selected_monitor.size().width,
                    screen_height: selected_monitor.size().height
                }.into_bytes();

                Task::perform(
                    async move {
                        let mut p2p_lock = p2p_arc.lock().await;
                        let p2p_ref = p2p_lock.as_mut().unwrap();
                        
                        let client_hello_bytes = p2p_ref.read().await.unwrap();
                        let client_hello_enum = match FromBytes::parse(&client_hello_bytes[..]) {
                            FromBytes::ClientHello(m) => m,
                            t => panic!("Expected client hello, received other bytes: {:?}", t)
                        };
                        
                        let _ = p2p_ref.send(&server_info_bytes).await;
                        
                        ()
                    },
                    Message::Connect
                )
            },

            Message::Disconnect => {
                self.pin = None;
                self.key = None;
                self.connected = false;
                self.error = String::from("");
                self.wait_reason = String::from("");
                let p2p_arc = self.p2p.clone();
                self.p2p = Arc::new(Mutex::new(None));

                Task::perform(
                async move {
                        let mut lock = p2p_arc.lock().await;
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

            Message::Connect(()) => {
                self.connected = true;
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
                        let monitor = &self.available_monitors[self.selected_monitor.unwrap()];
                        self.mouse.move_mouse(monitor.position().x as u32 + x, monitor.position().y as u32 + y);
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
}

struct P2PObject(Arc<Mutex<Option<p2p::P2P>>>);

impl Hash for P2PObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

fn p2p_stream(feed: &P2PObject) -> impl iced::futures::Stream<Item = Message> + use<> {
    let p2p = feed.0.clone();

    iced::futures::stream::unfold(p2p, |p2p| async move {
        let mut p2p_lock = p2p.lock().await;
        
        let p2p_ref = match p2p_lock.as_mut() {
            Some(t) => t,
            None => {
                drop(p2p_lock);
                return Some((Message::Null(()), p2p))
            }
        };

        let response = match p2p_ref.read().await {
            Ok(t) => t,
            Err(_) => {
                drop(p2p_lock);
                return Some((Message::Null(()), p2p))
            }
        };

        drop(p2p_lock);
        match FromBytes::parse(&response[..]) {
            FromBytes::MouseClick(t) => {
                return Some((Message::ClientMessage(ClientMessage::MouseClick(t.button, t.state)), p2p))
            },
            FromBytes::MouseMove(t) => {
                return Some((Message::ClientMessage(ClientMessage::MouseMove(t.x, t.y)), p2p))
            },
            FromBytes::UnknownInstruction(_) => {},
            // FromBytes::ClientInformation(t) => {},
            _ => {}
        }

        Some((Message::Null(()), p2p))
    })
}

pub fn subscription(window: &Window) -> Subscription<Message> {
    if window.connected {
        return iced::Subscription::run_with(P2PObject(window.p2p.clone()), p2p_stream);
    }
    return iced::Subscription::none();
}