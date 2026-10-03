use std::{ops::Add, sync::{Arc, Mutex}};

use iced::Task;
use crate::{p2p::protocol::{self, IntoBytes}, window::{self, Message, modules}};

pub struct Heartbeat {
    /// Time to live without receiving a heartbeat
    pub ttl: std::time::Duration,
    /// Time between messages
    pub delay: std::time::Duration,
    /// When the last heartbeat was received. None means no heartbeat has been received yet to prevent a time out before the first heartbeat
    pub last_message: Option<std::time::Instant>,
}

impl Heartbeat {
    pub fn new(ttl: std::time::Duration, delay: std::time::Duration) -> Self {
        Self {
            ttl,
            delay,
            last_message: None
        }
    }
}

pub struct HeartbeatParameters(pub Arc<Mutex<Heartbeat>>);

impl std::hash::Hash for HeartbeatParameters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Force it to be a singleton
        "heartbeat".hash(state);
    }
}

pub fn stream(params: &HeartbeatParameters) -> impl iced::futures::Stream<Item = Message> + use<> {
    let parameters = params.0.clone();
    let delay = parameters.lock().unwrap().delay;

    iced::futures::stream::unfold((parameters, delay), |(parameters, delay)| async move {
        tokio::time::sleep(delay).await;

        let lock = parameters.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some(last_message) = &lock.last_message {
            if now > last_message.add(lock.ttl) {
                drop(lock);
                return Some((Message::HeartbeatMessage(HeartbeatMessage::Disconnect), (parameters, delay)))
            }
        }

        
        drop(lock);
        Some((Message::HeartbeatMessage(HeartbeatMessage::Send), (parameters, delay)))
    })
}

#[derive(Copy, Clone)]
pub enum HeartbeatMessage {
    Send,
    Disconnect
}

pub fn update(this: &mut window::Window, message: HeartbeatMessage) -> Task<Message> {
    match message {
        HeartbeatMessage::Send => {
            let p2p_clone = this.p2p.clone();
            Task::perform(async move {
                    let lock = p2p_clone.read().await;
                    let p2p = match lock.as_ref() {
                        Some(t) => t,
                        None => return ()
                    };

                    let hb = protocol::Heartbeat {};
                    let bytes = hb.into_bytes();

                    let _ = p2p.send(&bytes[..]).await;
                    
                    ()

                },
                Message::Empty
            )
        },
        HeartbeatMessage::Disconnect => {
            modules::host::update(this, modules::host::ConnectFlow::Disconnect).map(Message::HostConnectFlow)
        }
    }
}