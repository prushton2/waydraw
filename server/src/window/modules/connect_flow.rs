use std::sync::Arc;

use iced::Task;
use p2p::{self, protocol::{ClientHello, FromBytes, IntoBytes, ServerHello}};
use tokio::sync::RwLock;

use crate::{screen_grabber::ScreenCapture, window::Window};

#[derive(Clone)]
pub enum ConnectFlow {
    Register,
    AwaitClient(Result<(Arc<RwLock<Option<p2p::P2P>>>, String, String), String>),
    SendHello(Result<(), String>),
    Connect(ClientHello),
    Disconnect,
    Null(())
}

pub fn update(this: &mut Window, message: ConnectFlow) -> Task<ConnectFlow> {
    match message {
        ConnectFlow::Register => {
            if this.selected_monitor.is_none() {
                this.error = "Error: Please select a monitor".to_owned();
                return Task::none()
            }

            let key = match p2p::p2p::load_or_create_secret_key() {
                Ok(t) => t,
                Err(e) => {
                    this.error = e.to_string();
                    return Task::none();
                }
            };
            
            this.error = "".to_owned();
            this.wait_reason = "Registering...".to_owned();

            Task::perform(
                async move {
                    let (server, key) = p2p::P2P::init(key).await.map_err(|e| e.to_string())?;

                    let pin = p2p::remote_key_store::generate_pin();
                    let key = key.to_string();
                    
                    match p2p::remote_key_store::set(&pin, &key.to_string()).await {
                        Ok(_) => {},
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
                    this.error = t;
                    this.wait_reason = String::from("");
                    return Task::none()
                }
            };

            this.p2p = p2p;
            this.key = Some(key);
            this.pin = Some(pin);

            let arc = this.p2p.clone();
            
            this.wait_reason = "Waiting for connection".to_owned();
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
                this.error = format!("Error awaiting connection: {}", e);
                this.wait_reason = String::from("");
                return Task::none();
            }

            this.wait_reason = "Sending Hello".to_owned();
            let selected_monitor = &this.available_monitors[this.selected_monitor.unwrap_or(0)];

            // construct server info to send to client
            let p2p_arc = this.p2p.clone();

            let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>();

            let server_info = ServerHello {
                name: gethostname::gethostname().into_string().unwrap(),
                version: (version[0], version[1], version[2]),
                screen_width:  selected_monitor.resolution.0,
                screen_height: selected_monitor.resolution.1
            };

            let server_info_bytes = server_info.into_bytes();

            let pin = this.pin.clone().unwrap_or(String::from(""));

            Task::perform(
                async move {
                    p2p::remote_key_store::delete(&pin).await;

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
                ConnectFlow::Connect
            )
        },

        ConnectFlow::Connect(client_hello) => {
            let version = env!("CARGO_PKG_VERSION").split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>();

            if client_hello.version.0 != version[0] {
                this.error = format!("Incompatible versions: Client {}.{}.{} and Server {}.{}.{}. Please update each app to the same major version.", client_hello.version.0, client_hello.version.1, client_hello.version.2, version[0], version[1], version[2]);
                this.p2p = Arc::new(RwLock::new(None));
                this.pin = None;
                this.key = None;
                return Task::none();
            }

            *this.client_window_size.lock().unwrap() = (client_hello.window_width, client_hello.window_height);
            this.connected = true;

            let monitor_id = this.available_monitors[this.selected_monitor.unwrap()].id.clone();

            let screencap = ScreenCapture::new(&monitor_id).unwrap();

            *this.recording.write().unwrap() = Some(screencap);

            Task::none()
        },

        ConnectFlow::Disconnect => {
            this.pin = None;
            this.key = None;
            this.connected = false;
            this.recording.read().unwrap().as_ref().unwrap().kill();
            *this.recording.write().unwrap() = None;
            this.error = String::from("");
            this.wait_reason = String::from("");
            let p2p_arc = this.p2p.clone();
            this.p2p = Arc::new(RwLock::new(None));

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
