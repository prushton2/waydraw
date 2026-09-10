use std::sync::Arc;

use iroh::EndpointId;

use p2p::p2p::P2PError;
use p2p::protocol::{ServerHello, ClientHello};
use tokio::sync::Mutex;

use iced::Alignment::Center;
use iced::{Length::Fill, Subscription, Task, window};
use iced::widget::{self, button, column, container, row, space, text, text_input};

use p2p::{self, protocol::{self, IntoBytes}};
use p2p::protocol::mouse_click::{MouseButton, MouseState};


#[derive(Default)]
pub struct Window {
    server_info: Option<protocol::ServerHello>,
    p2p: Option<Arc<Mutex<p2p::P2P>>>,

    known_size: (usize, usize),
    
    pin_textbox: String,
    key_textbox: String,

    wait_reason: String,
    error: String,
}

#[derive(Clone)]
pub enum Message {
    MouseMove(f32, f32),
    MouseClick(MouseButton, MouseState),
    WindowResized((usize, usize)),

    PinSubmitted,
    P2PCreated(Result<(Arc<Mutex<p2p::P2P>>, protocol::ServerHello), P2PError>),
    
    PINTextbox(String),
    KeyTextbox(String),
    Sent(Result<(), String>)
}

impl Window {
    pub fn boot() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::MouseMove(x, y) => {
                let server_info = match self.server_info {
                    Some(t) => t,
                    None => return Task::none()
                };

                let p2p = match &self.p2p {
                    Some(t) => t.clone(),
                    None => return Task::none()
                };

                let mouse_pct = (x / self.known_size.0 as f32, y / self.known_size.1 as f32);
                let scaled_mouse_pos = (mouse_pct.0 * server_info.screen_width as f32, mouse_pct.1 * server_info.screen_height as f32);

                let message = p2p::protocol::MouseMove {x: scaled_mouse_pos.0 as u32, y: scaled_mouse_pos.1 as u32};

                Task::perform(
                async move {
                        match p2p.lock().await.send(&message.into_bytes()).await {
                            Ok(_) => Ok(()),
                            Err(t) => Err(format!("{:?}", t))
                        }
                    },
                    Message::Sent,
                )
            }
            Message::MouseClick(button, state) => {
                let p2p = match &self.p2p {
                    Some(t) => t.clone(),
                    None => return Task::none()
                };

                let message = p2p::protocol::MouseClick { button: button, state: state };

                Task::perform(
                async move {
                        match p2p.lock().await.send(&message.into_bytes()).await {
                            Ok(_) => Ok(()),
                            Err(t) => Err(format!("{:?} ", t))
                        }
                    },
                    Message::Sent,
                )
            },
            Message::PinSubmitted => {
                let pin_textbox = self.pin_textbox.clone();
                let key_textbox = self.key_textbox.clone();
                self.wait_reason = String::from("Connecting to server...");
                self.error = String::from("");

                let known_size_clone = self.known_size.clone();

                Task::perform(
                    async move {
                        let key;
                        if key_textbox.len() == 0 {
                            key = p2p::remote_key_store::get(&pin_textbox).await;
                            p2p::remote_key_store::delete(&pin_textbox).await;
                        } else {
                            key = key_textbox;
                        }

                        let parsed_key = key.parse::<EndpointId>().map_err(|_| P2PError::during("Error reading key", P2PError::InputError("Invalid key or pin".to_string())))?;

                        let mut client = p2p::P2P::connect(parsed_key).await?;
                        
                        let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

                        let client_hello = ClientHello {
                            version: (version[0], version[1], version[2]),
                            window_width:  known_size_clone.0 as u32,
                            window_height: known_size_clone.1 as u32
                        };

                        let _ = client.send(&client_hello.into_bytes()).await;
                        
                        let data = client.read().await?;

                        let server_info = match protocol::FromBytes::parse(&data) {
                            protocol::FromBytes::ServerHello(d) => d,
                            _ => panic!("Did not receive server info")
                        };
                        
                        let p2p: Arc<Mutex<p2p::P2P>> = Arc::new(Mutex::new(client));

                        Ok((p2p, server_info))
                    },
                    Message::P2PCreated,
                )
            },

            Message::P2PCreated(result) => {
                self.wait_reason = String::from("");

                let (p2p, server_info) = match result {
                    Ok(t) => t,
                    Err(e) => {
                        self.error = String::from(e);
                        return Task::none()
                    }
                };

                self.p2p = Some(p2p);
                self.server_info = Some(server_info);
                Task::none()
            },

            Message::WindowResized(size) => {
                self.known_size = size;
                Task::none()
            },
            Message::Sent(result) => {
                if let Err(_) = result { 
                    self.p2p = None;
                    self.error = String::from("");
                    self.wait_reason = String::from("");
                    self.key_textbox = String::from("");
                    self.pin_textbox = String::from("");
                }
                
                Task::none()
            },

            Message::PINTextbox(f) => {
                self.pin_textbox = f;
                Task::none()
            },
            Message::KeyTextbox(f) => {
                self.key_textbox = f;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.p2p.is_none() {
            return container (
                column![
                    text("Input device pin").width(Fill).align_x(Center),
                    row![text_input("000000", &self.pin_textbox).on_input(Message::PINTextbox), space().width(20), button("Connect").on_press(Message::PinSubmitted)],
                    text("OR").width(Fill).align_x(Center),
                    text("Input device key").width(Fill).align_x(Center),
                    row![text_input("", &self.key_textbox).on_input(Message::KeyTextbox), space().width(20), button("Connect").on_press(Message::PinSubmitted)],
                    space().height(20),
                    text(&self.wait_reason).width(Fill).align_x(Center),
                    text(&self.error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
                ]
                .max_width(400)
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        }

        return container (
            widget::MouseArea::new(
                widget::row![]
                .width(Fill)
                .height(Fill)
            )
            .on_move(|point| {return Message::MouseMove(point.x, point.y)})

            .on_press        (Message::MouseClick(MouseButton::Left,  MouseState::Pressed ))
            .on_release      (Message::MouseClick(MouseButton::Left,  MouseState::Released))
            .on_right_press  (Message::MouseClick(MouseButton::Right, MouseState::Pressed ))
            .on_right_release(Message::MouseClick(MouseButton::Right, MouseState::Released))

        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}

pub fn subscription(_: &Window) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized((size.width as usize, size.height as usize)))
}