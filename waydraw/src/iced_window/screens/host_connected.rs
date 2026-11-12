use std::hash::{Hash, Hasher};

use iced::futures::channel::mpsc::{self, Receiver, Sender};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Length, Task};
use iroh::SecretKey;

use crate::encoding::{self, Codec};
use crate::iced_window::screens::host::Monitor;
use crate::iced_window::{Message, screens, styles};
use crate::input::{self, Input};
use crate::p2p::protocol::{FromBytes, IntoBytes};
use crate::p2p::{self, protocol}; // for out.send(..).await and cmd_rx.next().await

#[derive(Clone)]
pub struct HostConnected {
    pub key: String,
    pub pin: String,
    pub error: String,
    pub sender: Option<Sender<WorkerEvent>>,
    pub client_info: Option<protocol::ClientHello>,
    pub monitor: Monitor,
}

impl HostConnected {
    pub fn new(monitor: Monitor) -> Self {
        Self {
            key: String::from(""),
            pin: String::from(""),
            error: String::from(""),
            sender: None,
            client_info: None,
            monitor,
        }
    }

    pub fn update(&mut self, message: HostConnectedMessage) -> Task<Message> {
        match message {
            HostConnectedMessage::WorkerEvent(event) => {
                match event {
                    WorkerEvent::UpdateKnownHosts(hostname, key) => return Task::done(Message::UpdateKnownHosts(hostname, key)),
                    WorkerEvent::Disconnect(reason) => return Task::done(Message::HostConnectedMessage(HostConnectedMessage::Disconnect(reason))),
                    WorkerEvent::Ready(t) => self.sender = Some(t),
                    WorkerEvent::UpdateKey(key) => self.key = key,
                    WorkerEvent::UpdatePin(pin) => self.pin = pin,
                    WorkerEvent::Failed(e) => self.error = e,
                    WorkerEvent::SetClientInfo(t) => self.client_info = Some(t),
                    _ => {}
                }
                Task::none()
            }
            HostConnectedMessage::Disconnect(reason) => {
                if let Some(sender) = &mut self.sender {
                    let _ = sender.try_send(WorkerEvent::Disconnect(reason));
                }
                Task::done(Message::Tab(crate::iced_window::Screen::Host(
                    screens::Host::new(),
                )))
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

        fn textbox_row<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Message> {
            row![
                text(label)
                    .width(Length::Fixed(80.0))
                    .style(text::secondary),
                text_input("", value).style(|t, _| styles::context_textbox(t)),
            ]
            .spacing(12)
            .into()
        }

        let mut card = column![text("Connected").size(22)]
            .spacing(8)
            .align_x(Alignment::Start);

        card = card.push(textbox_row("Pin", &self.pin));
        card = card.push(textbox_row("Key", &self.key));

        if let Some(client_info) = self.client_info.as_ref() {
            card = card
                .push(info_row("Device", deref(&client_info.host_name)))
                .push(info_row("OS", deref(&client_info.name)))
                .push(info_row("Version", deref(&client_info.os_version)))
                .push(info_row("Kernel", deref(&client_info.kernel_version)));
        } else {
            card = card.push(info_row("", "No device connected"))
        }

        card = card.push(
            container(
                button(text("Disconnect").align_x(Alignment::Center)).on_press(
                    Message::HostConnectedMessage(HostConnectedMessage::Disconnect(
                        "Host initiated disconnect".to_string(),
                    )),
                ),
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
    Disconnect(String),
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    // From Worker
    /// See Message::UpdateKnownHosts
    UpdateKnownHosts(String, String),
    Ready(mpsc::Sender<WorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetClientInfo(protocol::ClientHello),
    Disconnect(String),

    // To Worker
    Frame(Vec<u8>),
}

#[derive(Debug)]
pub enum IncomingEvent {
    WorkerEvent(WorkerEvent),
    P2p(FromBytes),
}

#[derive(Clone)]
pub struct Params {
    pub id: u64,
    pub secret_key: SecretKey,
    pub monitor: Monitor,
}

// Hash ONLY the session id. Anything that changes mid-session (window size etc.)
// must go through Command, or iced restarts the worker = new connection.
impl Hash for Params {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.id.hash(h)
    }
}

pub fn worker(params: &Params) -> impl Stream<Item = WorkerEvent> + use<> {
    let params = params.clone();
    iced::stream::channel(100, async move |mut out| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(WorkerEvent::Ready(cmd_tx)).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, mut codec) = match handshake(&params, &mut out).await {
            Ok(t) => t,
            Err(e) => {
                let _ = out.send(WorkerEvent::Failed(e)).await;
                return;
            }
        };

        let reason = session(&params, &mut p2p, codec.as_mut(), &mut out, &mut cmd_rx).await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(WorkerEvent::Disconnect(reason)).await;
    })
}

async fn handshake(
    params: &Params,
    out: &mut Sender<WorkerEvent>,
) -> Result<(p2p::P2P, Box<dyn Codec>), String> {
    // Register
    let (mut p2p, key) = p2p::P2P::init(params.secret_key.clone())
        .await
        .map_err(|e| e.to_string())?;

    let key = key.to_string();

    let pin = match p2p::remote_key_store::set(&key.to_string()).await {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    out.try_send(WorkerEvent::UpdateKey(key.clone())).map_err(|e| {
        format!(
            "TrySendError: Full: {}, Disconnected: {}",
            e.is_full(),
            e.is_disconnected()
        )
    })?;
    out.try_send(WorkerEvent::UpdatePin(pin)).map_err(|e| {
        format!(
            "TrySendError: Full: {}, Disconnected: {}",
            e.is_full(),
            e.is_disconnected()
        )
    })?;

    // Await connection

    p2p.await_connection().await?;

    // Await hello

    let data = p2p.read().await?;

    let client_info = match protocol::FromBytes::parse(&data) {
        protocol::FromBytes::ClientHello(d) => d,
        _ => panic!("Did not receive client info"),
    };

    out.try_send(WorkerEvent::UpdateKnownHosts(client_info.host_name.clone().unwrap_or(String::from("Unnamed Device")), key))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;

    // Find a codec match

    let codec = crate::encoding::get_compatible_codecs()
        .into_iter()
        .filter(|c| client_info.supported_codecs.contains(&c.to_string()))
        .next()
        .map_err(|| "No valid codec found")?;

    out.try_send(WorkerEvent::SetClientInfo(client_info))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;

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

    return Ok((p2p, codec_box));
}

async fn session(
    params: &Params,
    p2p: &mut p2p::P2P,
    codec: &mut dyn Codec,
    out: &mut Sender<WorkerEvent>,
    cmd_rx: &mut Receiver<WorkerEvent>,
) -> String {
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
                Ok(t) => IncomingEvent::P2p(FromBytes::parse(&t)),
                Err(e) => return e.to_string(),
            },
        };

        match event {
            IncomingEvent::WorkerEvent(WorkerEvent::Frame(bytes)) => {
                let encoded = codec.encode(&bytes, params.monitor.resolution);
                let frame = protocol::Screenshot {
                    bytes: encoded,
                    width: params.monitor.resolution.0 as usize,
                    height: params.monitor.resolution.1 as usize,
                };

                let _ = p2p.send(&frame.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(WorkerEvent::Disconnect(reason)) => {
                let goodbye = protocol::Goodbye {};
                let _ = p2p.send(&goodbye.into_bytes()).await;
                return reason;
            }
            IncomingEvent::P2p(FromBytes::Goodbye(_)) => {
                let _ = out.send(WorkerEvent::Disconnect("Client disconnected".to_string())).await;
            }
            IncomingEvent::P2p(FromBytes::MouseClick(e)) => input.click_mouse(e.button, e.state),
            IncomingEvent::P2p(FromBytes::MouseMove(e)) => input.move_mouse(e.x as i32, e.y as i32),
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
            None => Err(op()),
        }
    }
}

fn deref(option: &Option<String>) -> &str {
    option.as_deref().unwrap_or("Unknown")
}
