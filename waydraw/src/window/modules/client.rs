use std::sync::Arc;

use iced::Task;
use iroh::EndpointId;
use tokio::sync::RwLock;

use crate::{p2p::{self, p2p::P2PError, protocol::{self, IntoBytes}}, window::{Window, modules::ui_state::{self, UIState}}};

#[derive(Clone)]
pub enum ConnectFlow {
    PinSubmitted(String),
    KeySubmitted(String),
    Connected(Result<(Arc<RwLock<Option<crate::p2p::P2P>>>, protocol::ServerHello), P2PError>)
}

pub fn update(this: &mut Window, message: ConnectFlow) -> Task<ConnectFlow> {
    match message {
        ConnectFlow::PinSubmitted(pin) => {
            Task::perform(async move {
                    let mut key= String::from("");

                    for _ in 0..3 {
                        match p2p::remote_key_store::get(&pin).await {
                            Ok(t) => {
                                key = t;
                                break;
                            },
                            Err(e) => {
                                println!("Error: {}", e);
                                key = String::from("Timeout");
                                break;
                            }
                        }
                    }                        
                    key
                },
                ConnectFlow::KeySubmitted
            )
        }
        ConnectFlow::KeySubmitted(key) => {
            let wait_ref: &mut String;
            let error_ref: &mut String;

            if let UIState::Client { pin_input: _, key_input: _, wait, error } = &mut this.ui_state {
                wait_ref = wait;
                error_ref = error;
            } else {
                return Task::none()
            }

            if key == "Timeout" {
                *wait_ref = String::from("");
                *error_ref = String::from("Error connecting to pin server");
                return Task::none()
            }

            *wait_ref = String::from("Connecting to server...");
            *error_ref = String::from("");

            let window_size_clone = this.window_size.clone();

            Task::perform(
                async move {
                    let parsed_key = key.parse::<EndpointId>().map_err(|_| P2PError::during("Error reading key", P2PError::InputError("Invalid key or pin".to_string())))?;

                    let p2p = p2p::P2P::connect(parsed_key).await?;
                    
                    let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

                    let client_hello = protocol::ClientHello {
                        version: (version[0], version[1], version[2]),
                        window_width:  window_size_clone.0 as u32,
                        window_height: window_size_clone.1 as u32
                    };

                    let _ = p2p.send(&client_hello.into_bytes()).await;
                    
                    let data = p2p.read().await?;

                    let server_info = match protocol::FromBytes::parse(&data) {
                        protocol::FromBytes::ServerHello(d) => d,
                        _ => panic!("Did not receive server info")
                    };
                    
                    let p2p: Arc<RwLock<Option<p2p::P2P>>> = Arc::new(RwLock::new(Some(p2p)));

                    Ok((p2p, server_info))
                },
                ConnectFlow::Connected,
            )
        }
        ConnectFlow::Connected(result) => {
            let wait_ref: &mut String;
            let error_ref: &mut String;

            if let UIState::Client { pin_input: _, key_input: _, wait, error } = &mut this.ui_state {
                wait_ref = wait;
                error_ref = error;
            } else {
                return Task::none()
            }

            *wait_ref = String::from("");
            
            let (p2p, server_hello) = match result {
                Ok(t) => t,
                Err(e) => {
                    *error_ref = String::from(e);
                    return Task::none()
                }
            };

            let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

            if server_hello.version.0 != version[0] {
                *error_ref = format!("Incompatible versions: Server {}.{}.{} and Client {}.{}.{}. Please update each app to the same major version.", server_hello.version.0, server_hello.version.1, server_hello.version.2, version[0], version[1], version[2]);
                this.p2p = Arc::new(RwLock::new(None));
                return Task::none();
            }
            
            this.p2p = p2p;
            this.server_info = Some(server_hello);

            ui_state::update(this, ui_state::UIUpdate::Connect);

            Task::none()

        }
    }
}