use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use iced::{Alignment, Length, Task};
use iced::futures::channel::mpsc::{self, Receiver, Sender};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, column, container, row, text};
use iroh::SecretKey;

use crate::encoding::{self, Codec};
use crate::iced_window::Message;
use crate::input::{self, Input};
use crate::p2p::protocol::{FromBytes, IntoBytes};
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
            client_info: None
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
                    WorkerEvent::Disconnect(r) => println!("{:?}", r),
                    _ => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        fn info_row<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Message> {
            row![
                text(label)
                    .width(Length::Fixed(80.0))
                    .style(text::secondary),
                text(value),
            ]
            .spacing(12)
            .into()
        }

        let mut card = column![text("Connected").size(22)]
            .spacing(8)
            .align_x(Alignment::Start);

        card = card.push(info_row("Pin", &self.pin));
        card = card.push(info_row("Key", &self.key));

        if let Some(client_info) = self.client_info.as_ref() {
            card = card
                .push(info_row("Device", deref(&client_info.host_name)))
                .push(info_row("OS", deref(&client_info.name)))
                .push(info_row("Version", deref(&client_info.os_version)))
                .push(info_row("Kernel", deref(&client_info.kernel_version)));
        }

        card = card.push(
            container(
                button(text("Disconnect").align_x(Alignment::Center))
                    // .on_press(Message::Disconnect),
            )
            .center_x(Length::Fill)
            .padding(iced::Padding::ZERO.top(12)),
        );

        container(
            container(card)
                .padding(20)
                .max_width(420)
                .style(container::rounded_box),
        )
        .center(Length::Fill)
        .into()
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
    Disconnect(String),
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
        let (mut p2p, mut codec) = match handshake(&params, &mut out, &mut cmd_rx).await {
            Ok(t) => t,
            Err(e) => { let _ = out.send(WorkerEvent::Failed(e)).await; return; }
        };

        let reason = session(&mut p2p, codec.as_mut(), &mut out, &mut cmd_rx).await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(WorkerEvent::Disconnect(reason)).await;
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

async fn session(p2p: &mut p2p::P2P, codec: &mut dyn Codec, out: &mut Sender<WorkerEvent>, cmd_rx: &mut Receiver<WorkerEvent>) -> String {
    let mut input: Box<dyn Input> = Box::new(input::DummyInput::new());
    #[cfg(not(debug_assertions))]
    let mut input: Box<dyn Input> = Box::new(input::EnigoInput::new());
    
    loop {
        let event = tokio::select! {
            cmd = cmd_rx.next() => match cmd {
                Some(t) => IncomingEvent::WorkerEvent(t),
                None => break, // UI dropped the sender, stop the worker
            },
            cmd = p2p.read() => match cmd {
                Ok(t) => IncomingEvent::Bytes(t),
                Err(e) => return e.to_string(),
            },
        };

        match event {
        //     IncomingEvent::WorkerEvent(worker_event) => {
        //         let _ = out.send(worker_event).await;
        //     },
            IncomingEvent::Bytes(b) => {
                match FromBytes::parse(&b) {
                    FromBytes::MouseClick(e) => input.click_mouse(e.button, e.state),
                    FromBytes::MouseMove(e) => input.move_mouse(e.x as i32, e.y as i32),
                    _ => {}
                }
            },
        //     IncomingEvent::None => {}
            _ => {}
        }
    }
    String::from("Failsafe return")
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

fn deref(option: &Option<String>) -> &str {
    option.as_deref().unwrap_or("Unknown")
}