use std::{
    sync::{Arc, RwLock, atomic::AtomicBool},
    thread,
};

use iced::futures::channel::mpsc::Sender;

pub struct Heartbeat {
    /// Time to live without receiving a heartbeat
    pub ttl: std::time::Duration,
    active: Arc<AtomicBool>,
    expiry: Arc<RwLock<Option<std::time::Instant>>>,
    _thread_handle: std::thread::JoinHandle<()>,
}

impl Heartbeat {
    pub fn new<T: 'static + Send + Clone>(
        ttl: std::time::Duration,
        delay: std::time::Duration,
        sender: Sender<T>,
        expire_message: T,
        send_heartbeat_message: T,
    ) -> Self {
        let expiry = Arc::new(RwLock::new(None));
        let active = Arc::new(AtomicBool::new(true));
        Self {
            ttl,
            expiry: expiry.clone(),
            active: active.clone(),
            _thread_handle: Self::thread(
                expiry,
                sender,
                delay,
                expire_message,
                active,
                send_heartbeat_message,
            ),
        }
    }

    pub fn refresh(&self) {
        match self.expiry.write() {
            Ok(mut t) => *t = Some(std::time::Instant::now() + self.ttl),
            Err(t) => *(t.into_inner()) = Some(std::time::Instant::now() + self.ttl),
        };
    }

    fn thread<T: 'static + Send + Clone>(
        expiry: Arc<RwLock<Option<std::time::Instant>>>,
        mut sender: Sender<T>,
        delay: std::time::Duration,
        expire_message: T,
        active: Arc<AtomicBool>,
        send_heartbeat_message: T,
    ) -> std::thread::JoinHandle<()> {
        thread::spawn({
            move || {
                while active.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = sender.try_send(send_heartbeat_message.clone());

                    if let Ok(t) = expiry.read()
                        && let Some(expiry) = *t
                        && expiry < std::time::Instant::now()
                    {
                        let _ = sender.try_send(expire_message);
                        return;
                    }

                    std::thread::sleep(delay);
                }
            }
        })
    }
}

impl Drop for Heartbeat {
    fn drop(&mut self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
