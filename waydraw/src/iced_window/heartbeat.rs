use std::{sync::{Arc, RwLock}, thread};

use iced::futures::channel::mpsc::Sender;

use crate::{
    iced_window::{self, Message},
    p2p::protocol::{self, IntoBytes},
};
pub struct Heartbeat {
    /// Time to live without receiving a heartbeat
    pub ttl: std::time::Duration,
    
    expiry: Arc<RwLock<Option<std::time::Instant>>>,
    thread_handle: std::thread::JoinHandle<()>
}

impl Heartbeat {
    pub fn new<T: 'static + Send + Clone>(ttl: std::time::Duration, delay: std::time::Duration, sender: Sender<T>, expire_message: T, send_heartbeat_message: T) -> Self {
        let expiry = Arc::new(RwLock::new(None));
        Self {
            ttl,
            expiry: expiry.clone(),
            thread_handle: Self::thread(expiry, sender, delay, expire_message, send_heartbeat_message)
        }
    }

    pub fn refresh(&self) {
        match self.expiry.write() {
            Ok(mut t) => *t = Some(std::time::Instant::now() + self.ttl),
            Err(t) => *(t.into_inner()) = Some(std::time::Instant::now() + self.ttl),
        };
    }

    fn thread<T: 'static + Send + Clone>(expiry: Arc<RwLock<Option<std::time::Instant>>>, sender: Sender<T>, delay: std::time::Duration, expire_message: T, send_heartbeat_message: T) -> std::thread::JoinHandle<()> {

        thread::spawn({
            let expiry = expiry.clone();
            let mut sender = sender;
            move || {
                loop {
                    std::thread::sleep(delay);
                    let _ = sender.try_send(send_heartbeat_message.clone());

                    match expiry.read() {
                        Ok(t) => {
                            if let Some(expiry) = *t && expiry < std::time::Instant::now() {
                                let _ = sender.try_send(expire_message);
                                return ();
                            }
                        },
                        Err(_) => {}
                    }

                }
            }
        })
    }
}