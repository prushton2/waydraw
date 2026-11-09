use std::hash::{Hash, Hasher};

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iroh::SecretKey; // for out.send(..).await and cmd_rx.next().await

#[derive(Clone)]
pub struct HostConnected {
    pub sender: Option<mpsc::Sender<HostConnectedMessage>>
}

impl HostConnected {
    fn new() -> Self {
        Self {
            sender: None
        }
    }
}

#[derive(Clone)]
pub enum HostConnectedMessage {
    WorkerEvent
}

#[derive(Clone)]
pub enum Event {
    Ready(mpsc::Sender<_>)
}

#[derive(Clone)]
pub struct Params { pub id: u64, pub secret_key: SecretKey, pub monitor: String }

// Hash ONLY the session id. Anything that changes mid-session (window size etc.)
// must go through Command, or iced restarts the worker = new connection.
impl Hash for Params {
    fn hash<H: Hasher>(&self, h: &mut H) { self.id.hash(h) }
}


pub fn worker(params: &Params) -> impl Stream<Item = Event> + use<> {
    let params = params.clone();
    iced::stream::channel(100, async move |mut out| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(Event::Ready(cmd_tx)).await;

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        let (mut p2p, codec) = match handshake(&params, &mut out, &mut cmd_rx).await {
            Ok(t) => t,
            Err(e) => { let _ = out.send(Event::Failed(e)).await; return; }
        };

        // let reason = session(&p2p, &params.monitor, &codec, &mut out, &mut cmd_rx).await;

        p2p.close().await; // the borrow from session() has ended, so &mut is fine
        // let _ = out.send(Event::Disconnected(reason)).await;
    })
}