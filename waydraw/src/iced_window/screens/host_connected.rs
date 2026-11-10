use std::hash::{Hash, Hasher};

use iced::Task;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::widget::button;
use iroh::SecretKey;

use crate::iced_window::Message; // for out.send(..).await and cmd_rx.next().await

#[derive(Clone)]
pub struct HostConnected {
    pub sender: Option<mpsc::Sender<WorkerEvent>>
}

impl HostConnected {
    pub fn new() -> Self {
        Self {
            sender: None
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
                    _ => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        button("CLICK ME YIPPEE").on_press(Message::HostConnectedMessage(HostConnectedMessage::Button)).into()
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
    Button
}

#[derive(Clone)]
pub struct Params { pub id: u64, pub secret_key: SecretKey, pub monitor: String }

// Hash ONLY the session id. Anything that changes mid-session (window size etc.)
// must go through Command, or iced restarts the worker = new connection.
impl Hash for Params {
    fn hash<H: Hasher>(&self, h: &mut H) { self.id.hash(h) }
}


pub fn worker(params: &Params) -> impl Stream<Item = WorkerEvent> + use<> {
    // let params = params.clone();
    iced::stream::channel(100, async move |mut out| {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let _ = out.send(WorkerEvent::Ready(cmd_tx)).await;

        // loop {
        //     match cmd_rx.try_recv() {
        //         Ok(t) => println!("{:?}", t),
        //         Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
        //     }
            
        // }

        // Handshake: p2p is owned here, so &mut methods (await_connection) just work
        // let (mut p2p, codec) = match handshake(&params, &mut out, &mut cmd_rx).await {
        //     Ok(t) => t,
        //     Err(e) => { let _ = out.send(Event::Failed(e)).await; return; }
        // };

        // let reason = session(&p2p, &params.monitor, &codec, &mut out, &mut cmd_rx).await;

        // p2p.close().await; // the borrow from session() has ended, so &mut is fine
        // let _ = out.send(Event::Disconnected(reason)).await;
    })
}