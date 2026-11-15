use std::hash::{Hash, Hasher};

use iced::futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{self, Receiver, Sender},
};
use iroh::EndpointId;

use crate::{
    encoding::{self, Codec},
    iced_window::heartbeat::Heartbeat,
    input::{MouseButton, MouseState},
    p2p::{
        self,
        protocol::{self, FromBytes, IntoBytes},
    },
};

#[derive(Clone, Debug)]
pub enum OutgoingWorkerEvent {
    // Sent from worker
    /// See Message::UpdateKnownHosts
    UpdateKnownHosts(String, String),
    Ready(Sender<IncomingWorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetServerInfo(protocol::ServerHello),
    Disconnect(String),
    Screenshot(Vec<u8>, (usize, usize)),
}

#[derive(Clone, Debug)]
pub enum IncomingWorkerEvent {
    // Sent to worker
    SendWindowSizeToHost(usize, usize),
    MouseMove(i32, i32),
    MouseClick(MouseButton, MouseState),
    KeyEvent(crate::input::Key, crate::input::KeyState),
    Heartbeat,
    Disconnect(String),
}

#[derive(Debug)]
pub enum IncomingEvent {
    WorkerEvent(IncomingWorkerEvent),
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

pub fn worker(params: &Params) -> impl Stream<Item = OutgoingWorkerEvent> + use<> {
    let params = params.clone();
    iced::stream::channel(100, async move |mut out| {
        let (mut cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(OutgoingWorkerEvent::Ready(cmd_tx.clone())).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, mut codec) = match handshake(&params, &mut out).await {
            Ok(t) => t,
            Err(e) => {
                let _ = out.send(OutgoingWorkerEvent::Failed(e)).await;
                return;
            }
        };

        let reason = session(&mut p2p, codec.as_mut(), &mut out, &mut cmd_tx, &mut cmd_rx).await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(OutgoingWorkerEvent::Disconnect(reason)).await;
    })
}

async fn handshake(
    params: &Params,
    out: &mut Sender<OutgoingWorkerEvent>,
) -> Result<(p2p::P2P, Box<dyn Codec>), String> {
    // Register

    let key = if !params.pin.is_empty() {
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
        protocol::FromBytes::Goodbye(_) => return Err(String::from("Host refused connection")),
        _ => return Err("Did not receive server info".to_string()),
    };

    if crate::get_version().0 != server_info.version.0 {
        return Err(format!(
            "Could not connect to host. Client's major version ({}) does not match host's major version ({}). Update either app to the same major version",
            crate::get_version().0,
            server_info.version.0
        ));
    }

    let _ = out
        .send(OutgoingWorkerEvent::UpdateKnownHosts(
            server_info.name.clone(),
            key,
        ))
        .await;

    let codec_box = encoding::get_codec(&server_info.selected_codec)?;

    let _ = out
        .send(OutgoingWorkerEvent::SetServerInfo(server_info))
        .await;

    Ok((p2p, codec_box))
}

async fn session(
    p2p: &mut p2p::P2P,
    codec: &mut dyn Codec,
    out: &mut Sender<OutgoingWorkerEvent>,
    cmd_tx: &mut Sender<IncomingWorkerEvent>,
    cmd_rx: &mut Receiver<IncomingWorkerEvent>,
) -> String {
    let heartbeat = Heartbeat::new(
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(1),
        cmd_tx.clone(),
        IncomingWorkerEvent::Disconnect(String::from("Lost connection to host")),
        IncomingWorkerEvent::Heartbeat,
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
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::Disconnect(reason)) => {
                let goodbye = protocol::Goodbye {};
                let _ = p2p.send(&goodbye.into_bytes()).await;
                return reason;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::MouseClick(button, state)) => {
                let message = protocol::MouseClick { button, state };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::MouseMove(x, y)) => {
                let message = protocol::MouseMove {
                    x: x as u32,
                    y: y as u32,
                };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::SendWindowSizeToHost(w, h)) => {
                let message = protocol::WindowResized {
                    window_width: w as u32,
                    window_height: h as u32,
                };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::Heartbeat) => {
                let message = protocol::Heartbeat {};
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::KeyEvent(key, state)) => {
                let message = protocol::KeyEvent { key, state };
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::P2p(FromBytes::Screenshot(ss)) => {
                let decoded = codec.decode(&ss.bytes);
                let _ = out
                    .send(OutgoingWorkerEvent::Screenshot(
                        decoded,
                        (ss.width, ss.height),
                    ))
                    .await;
            }
            IncomingEvent::P2p(FromBytes::Goodbye(_)) => {
                let _ = out
                    .send(OutgoingWorkerEvent::Disconnect(
                        "Host disconnected".to_string(),
                    ))
                    .await;
            }
            IncomingEvent::P2p(FromBytes::Heartbeat(_)) => {
                heartbeat.refresh();
            }
            _ => {}
        }
    }
    String::from("Failsafe return")
}
