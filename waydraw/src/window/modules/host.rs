use std::sync::Arc;

use iced::Task;
use tokio::sync::RwLock;

use crate::{encoding, screen_capture};
use crate::window::Window;
use crate::p2p::{self, protocol::{ClientHello, FromBytes, IntoBytes, ServerHello}};
use crate::window::modules::ui_state::{self, UIState};

#[derive(Clone)]
pub enum ConnectFlow {
    Register,
    AwaitClient(Result<(Arc<RwLock<Option<p2p::P2P>>>, String, String), String>),
    SendHello(Result<(), String>),
    Connect((ClientHello, String)),
    Disconnect,
    Null(())
}

pub fn update(this: &mut Window, message: ConnectFlow) -> Task<ConnectFlow> {
    match message {
        ConnectFlow::Register => {
            if let UIState::Host { pin: _, key: _, wait: _, error: _ } = &mut this.ui_state {
                ui_state::update(this, ui_state::UIUpdate::UpdateError("".to_owned()));
                ui_state::update(this, ui_state::UIUpdate::UpdateWait("Registering...".to_owned()));
            } else {
                return Task::none()
            }

            if this.monitors.iter().filter(|e| e.id == this.selected_monitor).nth(0).is_none() {
                ui_state::update(this, ui_state::UIUpdate::UpdateError("Select a monitor".to_owned()));
                ui_state::update(this, ui_state::UIUpdate::UpdateWait("".to_owned()));
                return Task::none();
            }

            let key = this.config.secret_key.clone();

            Task::perform(
                async move {
                    let (server, key) = p2p::P2P::init(key).await.map_err(|e| e.to_string())?;

                    let key = key.to_string();
                    
                    let pin = match p2p::remote_key_store::set(&key.to_string()).await {
                        Ok(t) => t,
                        Err(e) => return Err(e)
                    };

                    Ok((Arc::new(RwLock::new(Some(server))), key, pin))
                },
                ConnectFlow::AwaitClient
            )
        },

        ConnectFlow::AwaitClient(result) => {
            let (p2p, key, pin) = match result {
                Ok(t) => t,
                Err(t) => {
                    ui_state::update(this, ui_state::UIUpdate::UpdateError(t));
                    ui_state::update(this, ui_state::UIUpdate::UpdateWait(String::from("")));
                    return Task::none()
                }
            };

            this.p2p = p2p;
            ui_state::update(this, ui_state::UIUpdate::UpdateKeyTextbox(key));
            ui_state::update(this, ui_state::UIUpdate::UpdatePinTextbox(pin));

            let arc = this.p2p.clone();
            
            ui_state::update(this, ui_state::UIUpdate::UpdateWait("Waiting for connection".to_owned()));
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
                ConnectFlow::SendHello,
            )
        },

        ConnectFlow::SendHello(result) => {
            if let Err(e) = result {
                ui_state::update(this, ui_state::UIUpdate::UpdateError(format!("Error awaiting connection: {}", e)));
                ui_state::update(this, ui_state::UIUpdate::UpdateWait(String::from("")));
                return Task::none();
            }

            ui_state::update(this, ui_state::UIUpdate::UpdateWait(String::from("Sending Hello")));

            // construct server info to send to client
            let p2p_arc = this.p2p.clone();

            let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>();

            let mut server_info = ServerHello {
                selected_codec: String::from(""),
                name: gethostname::gethostname().into_string().unwrap(),
                version: (version[0], version[1], version[2]),
                screen_width:  0,
                screen_height: 0
            };


            let mut pin_copy: String = String::from("");
            if let ui_state::UIState::Host { pin, key: _, wait: _, error: _ } = &this.ui_state {
                pin_copy = pin.clone();
            }

            Task::perform(
                async move {
                    p2p::remote_key_store::delete(&pin_copy).await;

                    let p2p_lock = p2p_arc.read().await;
                    let p2p_ref = p2p_lock.as_ref().unwrap();
                    
                    let client_hello_bytes = p2p_ref.read().await.unwrap();
                    let client_hello = match FromBytes::parse(&client_hello_bytes[..]) {
                        FromBytes::ClientHello(m) => m,
                        t => panic!("Expected client hello, received other bytes: {:?}", t)
                    };

                    let compatible_codecs = encoding::get_compatible_codecs();
                    let matched_codecs = client_hello.supported_codecs.iter().filter(|e| compatible_codecs.contains(&(**e).as_str())).collect::<Vec<&String>>();
                    
                    let selected_codec = matched_codecs[0].clone();

                    server_info.selected_codec = selected_codec.clone();
                    let server_info_bytes = server_info.into_bytes();
                    let _ = p2p_ref.send(&server_info_bytes).await;
                    
                    (client_hello, selected_codec)
                },
                ConnectFlow::Connect
            )
        },

        ConnectFlow::Connect((client_hello, selected_codec)) => {
            let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

            if client_hello.version.0 != version[0] {
                this.p2p = Arc::new(RwLock::new(None));
                ui_state::update(this, ui_state::UIUpdate::UpdateError(format!("Incompatible versions: Client {}.{}.{} and Server {}.{}.{}. Please update each app to the same major version.", client_hello.version.0, client_hello.version.1, client_hello.version.2, version[0], version[1], version[2])));
                ui_state::update(this, ui_state::UIUpdate::UpdateKeyTextbox(String::from("")));
                ui_state::update(this, ui_state::UIUpdate::UpdatePinTextbox(String::from("")));
                return Task::none();
            }

            this.client_info = Some(client_hello);
            ui_state::update(this, ui_state::UIUpdate::Connect);

            let recorder_arc = this.video_recorder.clone();
            let monitor_name = this.selected_monitor.clone();
            let mut encoder_lock = this.encoder.lock().unwrap();
            *encoder_lock = Some(encoding::get_codec(&selected_codec).unwrap());
            
            let mut lock = recorder_arc.write().unwrap();
            *lock = Some(screen_capture::ScreenCapture::new(&monitor_name).unwrap());
            
            Task::none()
        },

        ConnectFlow::Disconnect => {
            ui_state::update(this, ui_state::UIUpdate::Disconnect);

            let recorder_arc = this.video_recorder.clone();
            let p2p_arc = this.p2p.clone();
            this.p2p = Arc::new(RwLock::new(None));
            
            let mut lock = recorder_arc.write().unwrap();
            if let Some(recorder) = lock.as_ref() {
                recorder.kill()
            }
            *lock = None;

            let mut lock = this.heartbeat.lock().unwrap();
            lock.last_message = None;
            drop(lock);

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
                ConnectFlow::Null,
            )
        },
        ConnectFlow::Null(_) => {
            Task::none()
        }
    }
}
