use std::{
    hash::{Hash, Hasher}
};

use iced::futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{self, Receiver, Sender},
};
use iroh::SecretKey;

use crate::{encoding, input, p2p::protocol::FromBytes};
use crate::{
    encoding::Codec,
    iced_window::{heartbeat::Heartbeat, screens},
    p2p::{
        self,
        protocol::{self, IntoBytes},
    },
};

#[derive(Clone, Debug)]
pub enum OutgoingWorkerEvent {
    /// See Message::UpdateKnownHosts
    UpdateKnownHosts(String, String),
    Ready(Sender<IncomingWorkerEvent>),
    Failed(String),
    UpdatePin(String),
    UpdateKey(String),
    SetClientInfo(protocol::ClientHello),
    Disconnect(String),
    SetConfirmConnectPromptSender(Option<Sender<bool>>),
}

#[derive(Clone, Debug)]
pub enum IncomingWorkerEvent {
    Frame(Vec<u8>, (u32, u32)),
    Disconnect(String),
    Heartbeat,
}

#[derive(Clone, Debug)]
pub enum IncomingEvent {
    WorkerEvent(IncomingWorkerEvent),
    P2p(protocol::FromBytes),
}

#[derive(Clone, Debug)]
pub struct Params {
    pub id: u64,
    pub secret_key: SecretKey,
    pub monitor: screens::host::Monitor,
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
        let (mut cmd_tx, mut cmd_rx): (Sender<IncomingWorkerEvent>, Receiver<IncomingWorkerEvent>) =
            mpsc::channel(32);
        let _ = out.send(OutgoingWorkerEvent::Ready(cmd_tx.clone())).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, mut codec, mut client_info) = match handshake(&params, &mut out).await {
            Ok(t) => t,
            Err(e) => {
                let _ = out.send(OutgoingWorkerEvent::Failed(e)).await;
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
            &mut client_info,
        )
        .await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        let _ = out.send(OutgoingWorkerEvent::Disconnect(reason)).await;
    })
}

async fn handshake(
    params: &Params,
    out: &mut Sender<OutgoingWorkerEvent>,
) -> Result<(p2p::P2P, Box<dyn Codec>, protocol::ClientHello), String> {
    // Register
    let (mut p2p, key) = p2p::P2P::init(params.secret_key.clone())
        .await
        .map_err(|e| e.to_string())?;

    let key = key.to_string();

    let pin = match p2p::remote_key_store::set(&key.to_string()).await {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    out.try_send(OutgoingWorkerEvent::UpdateKey(key.clone()))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;
    out.try_send(OutgoingWorkerEvent::UpdatePin(pin.clone()))
        .map_err(|e| {
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
        _ => return Err("Did not receive client info".to_string()),
    };

    // Check Version

    if crate::get_version().0 != client_info.version.0 {
        return Err(format!(
            "Could not connect to client. Host's major version ({}) does not match client's major version ({}). Update either app to the same major version",
            crate::get_version().0,
            client_info.version.0
        ));
    }

    // Send client info to main thread

    out.try_send(OutgoingWorkerEvent::SetClientInfo(client_info.clone()))
        .map_err(|e| {
            format!(
                "TrySendError: Full: {}, Disconnected: {}",
                e.is_full(),
                e.is_disconnected()
            )
        })?;

    // Confirm Prompt

    let (sender, mut receiver) = mpsc::channel(4);

    out.try_send(OutgoingWorkerEvent::SetConfirmConnectPromptSender(Some(
        sender,
    )))
    .map_err(|e| {
        format!(
            "TrySendError: Full: {}, Disconnected: {}",
            e.is_full(),
            e.is_disconnected()
        )
    })?;

    // Also read the p2p here to disconnect the host if the client closes the app or disconnects
    let confirmed = tokio::select! {
        answer = receiver.recv() => answer.unwrap_or(false),
        msg = p2p.read() => {
            let _ = out.send(OutgoingWorkerEvent::SetConfirmConnectPromptSender(None)).await;
            return Err(match msg {
                Ok(_) => "Client cancelled the connection".to_string(),
                Err(e) => e.to_string(),
            });
        }
    };

    if !confirmed {
        let goodbye = protocol::Goodbye {};
        let _res = p2p.send(&goodbye.into_bytes()).await;
        p2p.close().await;
        return Err(String::from("Host denied connection"));
    }

    let _ = out
        .send(OutgoingWorkerEvent::SetConfirmConnectPromptSender(None))
        .await;
    drop(receiver); // redundant, but this has the same vibe as a null pointer, so im dropping it.

    // Find a codec match

    let codec = crate::encoding::get_compatible_codecs()
        .into_iter()
        .find(|c| client_info.supported_codecs.contains(&c.to_string()))
        .map_err(|| "No valid codec found")?;

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

    p2p::remote_key_store::delete(&pin).await;

    Ok((p2p, codec_box, client_info))
}

async fn session(
    params: &Params,
    p2p: &mut p2p::P2P,
    codec: &mut dyn Codec,
    out: &mut Sender<OutgoingWorkerEvent>,
    cmd_tx: &mut Sender<IncomingWorkerEvent>,
    cmd_rx: &mut Receiver<IncomingWorkerEvent>,
    client_info: &mut protocol::ClientHello,
) -> String {
    #[cfg(debug_assertions)]
    let mut input: Box<dyn input::Input> = Box::new(input::DummyInput::new());
    #[cfg(not(debug_assertions))]
    let mut input: Box<dyn input::Input> = Box::new(input::EnigoInput::new());

    let heartbeat = Heartbeat::new(
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(1),
        cmd_tx.clone(),
        IncomingWorkerEvent::Disconnect(String::from("Lost connection to client")),
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
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::Frame(bytes, res)) => {
                let encoded = codec.encode(&bytes, res);
                let frame = protocol::Screenshot {
                    bytes: encoded,
                    width: res.0 as usize,
                    height: res.1 as usize,
                };

                let _ = p2p.send(&frame.into_bytes()).await;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::Disconnect(reason)) => {
                let goodbye = protocol::Goodbye {};
                let _ = p2p.send(&goodbye.into_bytes()).await;
                return reason;
            }
            IncomingEvent::WorkerEvent(IncomingWorkerEvent::Heartbeat) => {
                let message = protocol::Heartbeat {};
                let _ = p2p.send(&message.into_bytes()).await;
            }
            IncomingEvent::P2p(FromBytes::Goodbye(_)) => {
                let _ = out
                    .send(OutgoingWorkerEvent::Disconnect(
                        "Client disconnected".to_string(),
                    ))
                    .await;
            }
            IncomingEvent::P2p(FromBytes::MouseClick(e)) => input.click_mouse(e.button, e.state),
            IncomingEvent::P2p(FromBytes::MouseMove(protocol::MouseMove { x, y })) => {
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
            IncomingEvent::P2p(FromBytes::WindowResized(protocol::WindowResized{window_width: w, window_height: h})) => {
                client_info.window_width = w;
                client_info.window_height = h;
            }
            IncomingEvent::P2p(_) => {}
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
