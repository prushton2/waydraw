use std::hash::{Hash, Hasher};
use std::sync::Arc;

use iced::futures::channel::mpsc::{self, Receiver, Sender};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Length, Task};
use iroh::SecretKey;

use crate::encoding::{self, Codec};
use crate::iced_window::heartbeat::Heartbeat;
use crate::iced_window::screens::host::Monitor;
use crate::iced_window::{Message, Screen, styles};
use crate::input::{self, Input};
use crate::p2p::protocol::{ClientHello, FromBytes, IntoBytes, MouseMove};
use crate::p2p::{self, protocol}; // for out.send(..).await and cmd_rx.next().await

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
                    WorkerEvent::UpdateKnownHosts(hostname, key) => {
                        return Task::done(Message::UpdateKnownHosts(hostname, key));
                    }
                    WorkerEvent::Disconnect(reason) => {
                        return Task::done(Message::HostConnectedMessage(
                            HostConnectedMessage::Disconnect(reason),
                        ));
                    }
                    WorkerEvent::UpdateKey(key) => self.key = key,
                    WorkerEvent::UpdatePin(pin) => self.pin = pin,
                    WorkerEvent::Failed(e) => self.error = e,
                    WorkerEvent::SetClientInfo(t) => self.client_info = Some(t),
                    WorkerEvent::Ready(t) => self.sender = Some(t),
                    _ => {}
                }
                Task::none()
            }
            HostConnectedMessage::Disconnect(reason) => {
                if let Some(sender) = &mut self.sender {
                    let _ = sender.try_send(WorkerEvent::Disconnect(reason));
                }
                Task::done(Message::Tab(Arc::new(|| Screen::Host(super::Host::new()))))
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

        let mut card = column![].spacing(8).align_x(Alignment::Start);

        if let Some(client_info) = self.client_info.as_ref() {
            card = card
                .push(text("Connected").size(22))
                .push(info_row("Device", deref(&client_info.host_name)))
                .push(info_row("OS", deref(&client_info.name)))
                .push(info_row("Version", deref(&client_info.os_version)))
                .push(info_row("Kernel", deref(&client_info.kernel_version)));
        } else if !self.error.is_empty() {
            card = card
                .push(text("Error Connecting").size(22))
                .push(text(&self.error).style(text::danger));
        } else if self.key.is_empty() {
            card = card
                .push(text("Registering...").size(22))
                .push(textbox_row("Pin", &self.pin))
                .push(textbox_row("Key", &self.key))
                .push(text(&self.error).style(text::danger));
        } else {
            card = card
                .push(text("Awaiting Connection").size(22))
                .push(textbox_row("Pin", &self.pin))
                .push(textbox_row("Key", &self.key))
                .push(text(&self.error).style(text::danger));
        }

        card = card.push(
            container(
                button(text(if self.error.is_empty() { "Disconnect" } else { "Retry" }).align_x(Alignment::Center)).on_press(
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
    Heartbeat,
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
        let (mut cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(WorkerEvent::Ready(cmd_tx.clone())).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, mut codec, client_info) = match handshake(&params, &mut out).await {
            Ok(t) => t,
            Err(e) => {
                let _ = out.send(WorkerEvent::Failed(e)).await;
                return;
            }
        };

        let reason = session(
            &params,
            &mut p2p,
            codec.as_mut(),
            &mut out,
            &mut cmd_tx,
            &mut cmd_rx,
            &client_info,
        )
        .await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(WorkerEvent::Disconnect(reason)).await;
    })
}

async fn handshake(
    params: &Params,
    out: &mut Sender<WorkerEvent>,
) -> Result<(p2p::P2P, Box<dyn Codec>, ClientHello), String> {
    // Register
    let (mut p2p, key) = p2p::P2P::init(params.secret_key.clone())
        .await
        .map_err(|e| e.to_string())?;

    let key = key.to_string();

    let pin = match p2p::remote_key_store::set(&key.to_string()).await {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    out.try_send(WorkerEvent::UpdateKey(key.clone()))
        .map_err(|e| {
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

    out.try_send(WorkerEvent::UpdateKnownHosts(
        client_info
            .host_name
            .clone()
            .unwrap_or(String::from("Unnamed Device")),
        key,
    ))
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
        .find(|c| client_info.supported_codecs.contains(&c.to_string()))
        .map_err(|| "No valid codec found")?;

    out.try_send(WorkerEvent::SetClientInfo(client_info.clone()))
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

    Ok((p2p, codec_box, client_info))
}

async fn session(
    params: &Params,
    p2p: &mut p2p::P2P,
    codec: &mut dyn Codec,
    out: &mut Sender<WorkerEvent>,
    cmd_tx: &mut Sender<WorkerEvent>,
    cmd_rx: &mut Receiver<WorkerEvent>,
    client_info: &ClientHello,
) -> String {
    #[cfg(debug_assertions)]
    let mut input: Box<dyn Input> = Box::new(input::DummyInput::new());
    #[cfg(not(debug_assertions))]
    let mut input: Box<dyn Input> = Box::new(input::EnigoInput::new());

    let heartbeat = Heartbeat::new(
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(1),
        cmd_tx.clone(),
        WorkerEvent::Disconnect(String::from("Lost connection to client")),
        WorkerEvent::Heartbeat,
    );

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
            IncomingEvent::WorkerEvent(WorkerEvent::Heartbeat) => {
                let message = protocol::Heartbeat {};
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::P2p(FromBytes::Goodbye(_)) => {
                let _ = out
                    .send(WorkerEvent::Disconnect("Client disconnected".to_string()))
                    .await;
            }
            IncomingEvent::P2p(FromBytes::MouseClick(e)) => input.click_mouse(e.button, e.state),
            IncomingEvent::P2p(FromBytes::MouseMove(MouseMove { x, y })) => {
                let (offset_x, offset_y) = params.monitor.position;
                let client_res = (client_info.window_width, client_info.window_height);
                let scale = params.monitor.scale;

                let mouse_pct = (
                    x as f32 / client_res.0 as f32,
                    y as f32 / client_res.1 as f32,
                );

                let mouse_scaled_to_target_res = (
                    mouse_pct.0 * params.monitor.resolution.0 as f32,
                    mouse_pct.1 * params.monitor.resolution.1 as f32,
                );

                let mouse_position = (
                    ((mouse_scaled_to_target_res.0 as i32 + offset_x) as f32) / scale,
                    ((mouse_scaled_to_target_res.1 as i32 + offset_y) as f32) / scale,
                );

                input.move_mouse(mouse_position.0 as i32, mouse_position.1 as i32);
            }
            IncomingEvent::P2p(FromBytes::KeyEvent(e)) => {
                input.key_event(e.key, e.state);
            }
            IncomingEvent::P2p(FromBytes::Heartbeat(_)) => {
                heartbeat.refresh();
            }
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
