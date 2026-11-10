use std::hash::{Hash, Hasher};

use iced::Task;
use iced::futures::channel::mpsc::{self, Receiver, Sender};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{column, button, text};
use iroh::SecretKey;

use crate::encoding::{self, Codec};
use crate::iced_window::Message;
use crate::p2p::protocol::IntoBytes;
use crate::p2p::{self, protocol}; // for out.send(..).await and cmd_rx.next().await

#[derive(Clone)]
pub struct HostConnected {
    pub key: String,
    pub pin: String,
    pub sender: Option<Sender<WorkerEvent>>,
    pub client_info: Option<protocol::ClientHello>
}

impl HostConnected {
    pub fn new() -> Self {
        Self {
            key: String::from(""),
            pin: String::from(""),
            sender: None,
            client_info: None,
        }
    }

    pub fn update(&mut self, message: HostConnectedMessage) -> Task<Message> {
        match message {
            HostConnectedMessage::Button => {
                if let Some(sender) = &mut self.sender {
                    println!("Send: {:?}", sender.try_send(WorkerEvent::Button));
                } else {
                    println!("No sender")
                }
                Task::none()
            },
            HostConnectedMessage::WorkerEvent(event) => {
                match event {
                    WorkerEvent::Ready(t) => self.sender = Some(t),
                    WorkerEvent::UpdateKey(key) => self.key = key,
                    WorkerEvent::UpdatePin(pin) => self.pin = pin,
                    WorkerEvent::Failed(e) => println!("Worker failed: {}", e),
                    WorkerEvent::SetClientInfo(t) => self.client_info = Some(t),
                    _ => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            text(&self.pin),
            text(&self.key),
            button("CLICK ME YIPPEE").on_press(Message::HostConnectedMessage(HostConnectedMessage::Button))
        ].into()
    }
}

#[derive(Clone)]
pub enum HostConnectedMessage {
    WorkerEvent(WorkerEvent),
    Button,
}


#[derive(Clone, Debug)]
pub enum WorkerEvent {
    Ready(mpsc::Sender<WorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetClientInfo(protocol::ClientHello),
    Button
}

#[derive(Debug)]
pub enum IncomingEvent {
    None,
    WorkerEvent(WorkerEvent),
    Bytes(Vec<u8>)
}

#[derive(Clone)]
pub struct Params { pub id: u64, pub secret_key: SecretKey, pub monitor: String }

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
    let (mut p2p, key) = p2p::P2P::init(params.secret_key.clone()).await.map_err(|e| e.to_string())?;

    let key = key.to_string();

    let pin = match p2p::remote_key_store::set(&key.to_string()).await {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    out.try_send(WorkerEvent::UpdateKey(key)).map_err(|e| format!("TrySendError: Full: {}, Disconnected: {}", e.is_full(), e.is_disconnected()))?;
    out.try_send(WorkerEvent::UpdatePin(pin)).map_err(|e| format!("TrySendError: Full: {}, Disconnected: {}", e.is_full(), e.is_disconnected()))?;

    // Await connection

    p2p.await_connection().await?;

    // Await hello

    let data = p2p.read().await?;

    let client_info = match protocol::FromBytes::parse(&data) {
        protocol::FromBytes::ClientHello(d) => d,
        _ => panic!("Did not receive client info"),
    };

    // Find a codec match

    let codec = crate::encoding::get_compatible_codecs()
        .into_iter()
        .filter(|c| client_info.supported_codecs.contains(&c.to_string()))
        .next()
        .map_err(|| "No valid codec found")?;

    out.try_send(WorkerEvent::SetClientInfo(client_info)).map_err(|e| format!("TrySendError: Full: {}, Disconnected: {}", e.is_full(), e.is_disconnected()))?;

    // Send Hello

    let server_hello = protocol::ServerHello {
        selected_codec: String::from(codec),
        name: sysinfo::System::host_name().unwrap_or(String::from("Unnamed Device")),
        version: crate::get_version(),
        screen_width: 0,
        screen_height: 0,
    };

    p2p.send(&server_hello.into_bytes()).await?;

    let codec_box = encoding::get_codec(codec)?;

    return Ok((p2p, codec_box))
}

trait MapErr<T, E> {
    fn map_err(self, op: impl FnOnce() -> E) -> Result<T, E>;
}

impl<T, E> MapErr<T, E> for Option<T> {
    fn map_err(self, op: impl FnOnce() -> E) -> Result<T, E> {
        match self {
            Some(t) => Ok(t),
            None => Err(op())
        }
    }
}