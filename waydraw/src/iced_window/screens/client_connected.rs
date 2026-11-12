use fast_image_resize::Resizer;
use fast_image_resize::images::{Image, ImageRef};
use iced::Length;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::image;
use iced::{
    Task,
    futures::{
        Stream,
        channel::mpsc::{self, Receiver, Sender},
    },
    widget::{MouseArea, container, row, space, stack},
};
use iced_core::image::Allocation;
use iroh::EndpointId;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::encoding;
use crate::iced_window::Screen;
use crate::iced_window::heartbeat::Heartbeat;
use crate::input::{MouseButton, MouseState};
use crate::p2p::protocol::{self, FromBytes, IntoBytes};
use crate::{encoding::Codec, iced_window::{Message, screens}, p2p};

pub struct ClientConnected {
    pub key: String,
    pub pin: String,
    pub sender: Option<Sender<WorkerEvent>>,
    pub server_info: Option<protocol::ServerHello>,
    pub image_allocation: Option<iced_core::image::Allocation>,
}

impl ClientConnected {
    pub fn new(pin: String, key: String) -> Self {
        Self {
            key,
            pin,
            sender: None,
            server_info: None,
            image_allocation: None,
        }
    }

    pub fn from_key(key: String) -> Self {
        Self {
            key: key,
            pin: String::from(""),
            sender: None,
            server_info: None,
            image_allocation: None,
        }
    }

    pub fn from_pin(pin: String) -> Self {
        Self {
            key: String::from(""),
            pin: pin,
            sender: None,
            server_info: None,
            image_allocation: None,
        }
    }

    pub fn update(
        &mut self,
        message: ClientConnectedMessage,
        window_size: (usize, usize),
    ) -> Task<Message> {
        match message {
            ClientConnectedMessage::WorkerEvent(event) => {
                match event {
                    WorkerEvent::UpdateKnownHosts(hostname, key) => return Task::done(Message::UpdateKnownHosts(hostname, key)),
                    WorkerEvent::Disconnect(reason) => return Task::done(Message::ClientConnectedMessage(ClientConnectedMessage::Disconnect(reason))),
                    WorkerEvent::Ready(t) => self.sender = Some(t),
                    WorkerEvent::UpdateKey(key) => self.key = key,
                    WorkerEvent::UpdatePin(pin) => self.pin = pin,
                    WorkerEvent::Failed(e) => println!("Worker failed: {}", e),
                    WorkerEvent::SetServerInfo(e) => self.server_info = Some(e),
                    WorkerEvent::Screenshot(bytes, res) => {
                        if bytes.len() != res.0 * res.1 * 4 {
                            // Decoder produced no frame (e.g. waiting for a keyframe) or size mismatch
                            return Task::none();
                        }

                        // create resizer
                        let opts = fast_image_resize::ResizeOptions::new()
                            .resize_alg(fast_image_resize::ResizeAlg::Convolution(
                                fast_image_resize::FilterType::Bilinear,
                            ))
                            .use_alpha(false);

                        let src = ImageRef::new(
                            res.0 as u32,
                            res.1 as u32,
                            &bytes,
                            fast_image_resize::PixelType::U8x4,
                        )
                        .unwrap();

                        let mut dst = Image::new(
                            window_size.0 as u32,
                            window_size.1 as u32,
                            fast_image_resize::PixelType::U8x4,
                        );
                        let _ = Resizer::new().resize(&src, &mut dst, Some(&opts));

                        let dst_bytes = dst.into_vec();

                        let handle = image::Handle::from_rgba(
                            window_size.0 as u32,
                            window_size.1 as u32,
                            dst_bytes,
                        );

                        return image::allocate(handle).map(|e| {
                            Message::ClientConnectedMessage(ClientConnectedMessage::ImageAllocated(
                                e,
                            ))
                        });
                    }
                    _ => {}
                }
                Task::none()
            }
            ClientConnectedMessage::MouseClick(button, state) => {
                if let Some(sender) = &mut self.sender {
                    let mut sender_clone = sender.clone();
                    return Task::perform(
                        async move {
                            let _ = sender_clone
                                .send(WorkerEvent::MouseClick(button, state))
                                .await;
                        },
                        |_| Message::None,
                    );
                }
                Task::none()
            }
            ClientConnectedMessage::MouseMove(x, y) => {
                if let Some(sender) = &mut self.sender {
                    let mut sender_clone = sender.clone();
                    return Task::perform(
                        async move {
                            let _ = sender_clone.send(WorkerEvent::MouseMove(x, y)).await;
                        },
                        |_| Message::None,
                    );
                }
                Task::none()
            }
            ClientConnectedMessage::ImageAllocated(allocation) => {
                match allocation {
                    Ok(t) => self.image_allocation = Some(t),
                    Err(_) => {}
                }
                Task::none()
            }
            ClientConnectedMessage::Disconnect(reason) => {
                if let Some(sender) = &mut self.sender {
                    let _ = sender.try_send(WorkerEvent::Disconnect(reason));
                }
                Task::done(Message::Tab(
                    Arc::new(move || Screen::Client(screens::Client::new()))
                ))
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        container(stack![
            MouseArea::new(row![].width(Length::Fill).height(Length::Fill))
                .on_move(|point| Message::ClientConnectedMessage(
                    ClientConnectedMessage::MouseMove(point.x as i32, point.y as i32)
                ))
                .on_press(Message::ClientConnectedMessage(
                    ClientConnectedMessage::MouseClick(MouseButton::Left, MouseState::Pressed)
                ))
                .on_release(Message::ClientConnectedMessage(
                    ClientConnectedMessage::MouseClick(MouseButton::Left, MouseState::Released)
                ))
                .on_right_press(Message::ClientConnectedMessage(
                    ClientConnectedMessage::MouseClick(MouseButton::Right, MouseState::Pressed)
                ))
                .on_right_release(Message::ClientConnectedMessage(
                    ClientConnectedMessage::MouseClick(MouseButton::Right, MouseState::Released)
                )),
            match self.image_allocation.as_ref() {
                Some(allocation) => iced::Element::from(
                    image(allocation.handle())
                        .width(Length::Fill)
                        .height(Length::Fill)
                ),
                None => space().width(Length::Fill).height(Length::Fill).into(),
            },
            // space().width(Length::Fill).height(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Clone)]
pub enum ClientConnectedMessage {
    WorkerEvent(WorkerEvent),
    MouseClick(MouseButton, MouseState),
    MouseMove(i32, i32),
    ImageAllocated(Result<Allocation, image::Error>),
    Disconnect(String),
}

#[derive(Clone, Debug)]
pub enum WorkerEvent {
    // Sent from worker
    /// See Message::UpdateKnownHosts
    UpdateKnownHosts(String, String),
    Ready(Sender<WorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetServerInfo(protocol::ServerHello),
    Disconnect(String),

    // Sent to worker
    MouseMove(i32, i32),
    MouseClick(MouseButton, MouseState),
    Screenshot(Vec<u8>, (usize, usize)),
    Heartbeat
}

#[derive(Debug)]
pub enum IncomingEvent {
    WorkerEvent(WorkerEvent),
    P2p(FromBytes),
}

#[derive(Clone)]
pub struct Params {
    pub id: u64,
    pub pin: String,
    pub key: String,
    pub window_size: (usize, usize),
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
        let (mut p2p, mut codec) = match handshake(&params, &mut out).await {
            Ok(t) => t,
            Err(e) => {
                let _ = out.send(WorkerEvent::Failed(e)).await;
                return;
            }
        };

        let reason = session(&mut p2p, codec.as_mut(), &mut out, &mut cmd_tx, &mut cmd_rx).await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(WorkerEvent::Disconnect(reason)).await;
    })
}

async fn handshake(
    params: &Params,
    out: &mut Sender<WorkerEvent>,
) -> Result<(p2p::P2P, Box<dyn Codec>), String> {
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

    let p2p = p2p::P2P::connect(
        key.parse::<EndpointId>()
            .map_err(|e| format!("Error parsing key: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Error connecting: {:?}", e))?;

    // Send hello

    let client_hello = protocol::ClientHello {
        supported_codecs: encoding::get_compatible_codecs()
            .iter()
            .map(|e| e.to_string())
            .collect(),
        version: crate::get_version(),
        window_width: params.window_size.0 as u32,
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

    out.try_send(WorkerEvent::UpdateKnownHosts(server_info.name.clone(), key))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;

    let codec_box = encoding::get_codec(&server_info.selected_codec)?;

    out.try_send(WorkerEvent::SetServerInfo(server_info))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;

    return Ok((p2p, codec_box));
}

async fn session(
    p2p: &mut p2p::P2P,
    codec: &mut dyn Codec,
    out: &mut Sender<WorkerEvent>,
    cmd_tx: &mut Sender<WorkerEvent>,
    cmd_rx: &mut Receiver<WorkerEvent>,
) -> String {
    let heartbeat = Heartbeat::new(
        std::time::Duration::from_secs(5), 
        std::time::Duration::from_secs(1), 
        cmd_tx.clone(), 
        WorkerEvent::Disconnect(String::from("Lost connection to client")), 
        WorkerEvent::Heartbeat
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
            IncomingEvent::WorkerEvent(WorkerEvent::Disconnect(reason)) => {
                let goodbye = protocol::Goodbye {};
                let _ = p2p.send(&goodbye.into_bytes()).await;
                return reason;
            }
            IncomingEvent::WorkerEvent(WorkerEvent::MouseClick(button, state)) => {
                let message = protocol::MouseClick { button, state };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(WorkerEvent::MouseMove(x, y)) => {
                let message = protocol::MouseMove {
                    x: x as u32,
                    y: y as u32,
                };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(WorkerEvent::Heartbeat) => {
                let message = protocol::Heartbeat {};
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::P2p(FromBytes::Screenshot(ss)) => {
                let decoded = codec.decode(&ss.bytes);
                let _ = out
                    .send(WorkerEvent::Screenshot(decoded, (ss.width, ss.height)))
                    .await;
            }
            IncomingEvent::P2p(FromBytes::Goodbye(_)) => {
                let _ = out.send(WorkerEvent::Disconnect("Host disconnected".to_string())).await;
            }
            IncomingEvent::P2p(FromBytes::Heartbeat(_)) => {
                heartbeat.refresh();
            }
            _ => {}
        }
    }
    String::from("Failsafe return")
}
