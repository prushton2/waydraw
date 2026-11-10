use std::hash::{Hash, Hasher};
use iced::futures::{SinkExt, StreamExt};
use iced::{Task, futures::{Stream, channel::mpsc::{self, Receiver, Sender}}, widget::{column, text}};
use iroh::EndpointId;

use crate::encoding;
use crate::p2p::protocol::{self, IntoBytes};
use crate::{encoding::Codec, iced_window::Message, p2p};

#[derive(Clone)]

pub struct ClientConnected {
    pub key: String,
    pub pin: String,
    pub sender: Option<Sender<WorkerEvent>>,
    pub window_size: (usize, usize),
    pub server_info: Option<protocol::ServerHello>
}

impl ClientConnected {
    pub fn new(pin: String, key: String) -> Self {
        Self {
            key,
            pin,
            sender: None,
            window_size: (0, 0)
        }
    }

    pub fn update(&mut self, message: ClientConnectedMessage) -> Task<Message> {
        match message {
            ClientConnectedMessage::WorkerEvent(event) => {
                match event {
                    WorkerEvent::Ready(t) => self.sender = Some(t),
                    WorkerEvent::UpdateKey(key) => self.key = key,
                    WorkerEvent::UpdatePin(pin) => self.pin = pin,
                    WorkerEvent::Failed(e) => println!("Worker failed: {}", e),
                    WorkerEvent::SetServerInfo(e) => self.server_info = Some(e),
                    _ => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            text("Client connected")
        ].into()
    }
}

#[derive(Clone)]
pub enum ClientConnectedMessage {
    WorkerEvent(WorkerEvent),
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    Ready(Sender<WorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetServerInfo(protocol::ServerHello),
    Button
}

#[derive(Debug)]
pub enum IncomingEvent {
    None,
    WorkerEvent(WorkerEvent),
    Bytes(Vec<u8>)
}

#[derive(Clone)]
pub struct Params { pub id: u64, pub pin: String, pub key: String, pub window_size: (usize, usize) }

// Hash ONLY the session id. Anything that changes mid-session (window size etc.)
// must go through Command, or iced restarts the worker = new connection.
impl Hash for Params {
    fn hash<H: Hasher>(&self, h: &mut H) { self.id.hash(h) }
}


pub fn worker(params: &Params) -> impl Stream<Item = WorkerEvent> + use<> {
    let params = params.clone();
    iced::stream::channel(100, async move |mut out| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(WorkerEvent::Ready(cmd_tx)).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, codec) = match handshake(&params, &mut out, &mut cmd_rx).await {
            Ok(t) => t,
            Err(e) => { let _ = out.send(WorkerEvent::Failed(e)).await; return; }
        };

        loop {
            let response = tokio::select! {
                cmd = cmd_rx.next() => match cmd {
                    Some(t) => IncomingEvent::WorkerEvent(t),
                    None => break, // UI dropped the sender, stop the worker
                },
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => IncomingEvent::None,
            };

            println!("{:?}", response);
        }

        // let reason = session(&p2p, &params.monitor, &codec, &mut out, &mut cmd_rx).await;

        // p2p.close().await; // the borrow from session() has ended, so &mut is fine
        // let _ = out.send(Event::Disconnected(reason)).await;
    })
}

async fn handshake(params: &Params, out: &mut Sender<WorkerEvent>, cmd_rx: &mut Receiver<WorkerEvent>) -> Result<(p2p::P2P, Box<dyn Codec>), String> {
    // Register
    
    let key = if params.pin.len() != 0 {
        match p2p::remote_key_store::get(&params.pin.to_string()).await {
            Ok(t) => t,
            Err(e) => return Err(e),
        }
    } else {
        params.key.clone()
    };
    
    // Connect

    let p2p = p2p::P2P::connect(key.parse::<EndpointId>().unwrap()).await.map_err(|e| format!("{:?}", e))?;

    // Send hello

    let client_hello = protocol::ClientHello {
        supported_codecs: encoding::get_compatible_codecs()
            .iter()
            .map(|e| e.to_string())
            .collect(),
        version: crate::get_version(),
        window_width:  params.window_size.0 as u32,
        window_height: params.window_size.1 as u32,

        name: sysinfo::System::name(),
        kernel_version: sysinfo::System::kernel_version(),
        os_version: sysinfo::System::os_version(),
        host_name: sysinfo::System::host_name(),
    };

    p2p.send(&client_hello.into_bytes()).await?;

    // Await Hello

    let data = p2p.read().await?;

    let server_info = match protocol::FromBytes::parse(&data) {
        protocol::FromBytes::ServerHello(d) => d,
        _ => panic!("Did not receive server info"),
    };

    out.try_send(WorkerEvent::SetServerInfo(server_info)).map_err(|e| format!("TrySendError: Full: {}, Disconnected: {}", e.is_full(), e.is_disconnected()))?;


    return Err(String::from(""))
}