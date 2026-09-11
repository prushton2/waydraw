use std::sync::Arc;

use iroh::EndpointId;

use tokio::sync::RwLock;

use iced::Alignment::Center;
use iced::{Length::Fill, Task};
use iced::widget::{self, button, column, container, image, row, space, stack, text, text_input};

use p2p::p2p::P2PError;
use p2p::protocol::ClientHello;
use p2p::{self, protocol::{self, IntoBytes}};
use p2p::protocol::mouse_click::{MouseButton, MouseState};

use crate::window::ScreenshotType;

use super::{Window, Message};


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

                let p2p_arc = self.p2p.clone();

                let mouse_pct = (x / self.known_size.0 as f32, y / self.known_size.1 as f32);
                let scaled_mouse_pos = (mouse_pct.0 * server_info.screen_width as f32, mouse_pct.1 * server_info.screen_height as f32);

                let message = p2p::protocol::MouseMove {x: scaled_mouse_pos.0 as u32, y: scaled_mouse_pos.1 as u32};

                Task::perform(
                async move {
                    let temp = p2p_arc.read().await;
                    let lock = match temp.as_ref() {
                        Some(t) => t,
                        None => return Err(format!("No connection found"))
                    };

                    match lock.send(&message.into_bytes()).await {
                        Ok(_) => Ok(()),
                        Err(t) => Err(format!("{:?}", t))
                    }
                },
                    Message::Sent,
                )
            }
            Message::MouseClick(button, state) => {
                let p2p_arc = self.p2p.clone();

                let message = p2p::protocol::MouseClick { button: button, state: state };

                Task::perform(
                async move {
                    let temp = p2p_arc.read().await;
                    let lock = match temp.as_ref() {
                        Some(t) => t,
                        None => return Err(format!("No connection found"))
                    };

                    match lock.send(&message.into_bytes()).await {
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

                        let client = p2p::P2P::connect(parsed_key).await?;
                        
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
                        
                        let p2p: Arc<RwLock<Option<p2p::P2P>>> = Arc::new(RwLock::new(Some(client)));

                        Ok((p2p, server_info))
                    },
                    Message::P2PCreated,
                )
            },

            Message::P2PCreated(result) => {
                self.wait_reason = String::from("");
                
                let (p2p, server_hello) = match result {
                    Ok(t) => t,
                    Err(e) => {
                        self.error = String::from(e);
                        return Task::none()
                    }
                };

                let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

                if server_hello.version.0 != version[0] {
                    self.error = format!("Incompatible versions: Server {}.{}.{} and Client {}.{}.{}. Please update each app to the same major version.", server_hello.version.0, server_hello.version.1, server_hello.version.2, version[0], version[1], version[2]);
                    self.p2p = Arc::new(RwLock::new(None));
                    return Task::none();
                }
                
                self.p2p = p2p;
                self.server_info = Some(server_hello);
                self.connected = true;

                let pixels = vec![0 as u8; self.known_size.0 * self.known_size.1 * 4];
                self.handle = Some(image::Handle::from_rgba(self.known_size.0 as u32, self.known_size.1 as u32, pixels));

                Task::none()
            },

            Message::WindowResized(size) => {
                self.known_size = size;
                let p2p_arc = self.p2p.clone();

                let pixels = vec![0 as u8; self.known_size.0 * self.known_size.1 * 4];
                self.handle = Some(image::Handle::from_rgba(self.known_size.0 as u32, self.known_size.1 as u32, pixels));

                Task::perform(async move {
                    let window_resized_message = protocol::WindowResized {
                        window_width: size.0 as u32,
                        window_height: size.1 as u32,
                    };

                    let temp = p2p_arc.read().await;

                    let p2p = match temp.as_ref() {
                        Some(t) => t,
                        None => return ()
                    };

                    let _ = p2p.send(&window_resized_message.into_bytes()).await;

                    ()
                },
                    Message::Null
                )
            },
            Message::Sent(result) => {
                if let Err(_) = result { 
                    self.connected = false;
                    self.p2p = Arc::new(RwLock::new(None));
                    self.error = String::from("");
                    self.wait_reason = String::from("");
                    self.key_textbox = String::from("");
                    self.pin_textbox = String::from("");
                }
                
                Task::none()
            },

            Message::ScreenshotReceived(screenshot) => {
                let pixels: Vec<u8> = match screenshot {
                    ScreenshotType::Uncompressed(ss) => {
                        let mut pixels: Vec<u8> = vec![];
        
                        for pixel in ss.pixels {
                            pixels.push(pixel.0);
                            pixels.push(pixel.1);
                            pixels.push(pixel.2);
                            pixels.push(255);
                        }

                        pixels
                    },

                    ScreenshotType::Compressed(ss) => {
                        
                        // let params = brotli::enc::backward_references::BrotliEncoderParams::default();
                        let mut pixels: Vec<u8> = ss.bytes;
                        // let _ = brotli::BrotliDecompress(&mut ss.bytes.as_slice(), &mut pixels);


                        for i in 0..pixels.len() {
                            pixels.push(pixels[i]);
                            if i%3 == 2 {
                                pixels.push(255);
                            }
                        }

                        pixels
                    }
                };

                self.handle = Some(image::Handle::from_rgba(self.known_size.0 as u32, self.known_size.1 as u32, pixels));

                Task::none()
            },

            Message::PINTextbox(f) => {
                self.pin_textbox = f;
                Task::none()
            },
            Message::KeyTextbox(f) => {
                self.key_textbox = f;
                Task::none()
            },

            // Message::None => {
            //     Task::none()
            // },
            Message::Null(()) => {
                Task::none()
            },

        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if !self.connected {
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
            stack![
                widget::MouseArea::new(
                    widget::row![]
                    .width(Fill)
                    .height(Fill)
                )
                .on_move(|point| {return Message::MouseMove(point.x, point.y)})
    
                .on_press        (Message::MouseClick(MouseButton::Left,  MouseState::Pressed ))
                .on_release      (Message::MouseClick(MouseButton::Left,  MouseState::Released))
                .on_right_press  (Message::MouseClick(MouseButton::Right, MouseState::Pressed ))
                .on_right_release(Message::MouseClick(MouseButton::Right, MouseState::Released)),

                image(self.handle.as_ref().unwrap())
                    .width(Fill)
                    .height(Fill)
            ]

        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}