use std::{
    hash::Hash,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::RwLock;

use crate::{
    iced_window::Message,
    input::{Key, KeyState},
    p2p::{
        self,
        protocol::{self, IntoBytes},
    },
};

#[derive(Clone)]
pub struct KeySendStream {
    pub p2p: Arc<RwLock<Option<p2p::P2P>>>,
    pub queue: Arc<Mutex<Vec<(Key, KeyState)>>>,
}

impl Hash for KeySendStream {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Constant so iced keeps one stream alive. Two would race and reorder keys.
        "KeySendStream".hash(state);
    }
}

pub fn key_send_stream(
    params: &KeySendStream,
) -> impl iced::futures::Stream<Item = Message> + use<> {
    let params_clone = params.clone();

    iced::futures::stream::unfold(params_clone, |parameters| async move {
        loop {
            // Own statement: the std Mutex guard must not be held across an await
            let keys = std::mem::take(&mut *parameters.queue.lock().unwrap());

            if keys.is_empty() {
                // Wait here instead of yielding, every yielded Message runs update + view
                tokio::time::sleep(Duration::from_millis(5)).await;
                continue;
            }

            if let Some(p2p) = parameters.p2p.read().await.as_ref() {
                for (key, state) in keys {
                    let message = protocol::KeyEvent { key, state };
                    let _ = p2p.send(&message.into_bytes()).await;
                }
            }

            return Some((Message::None, parameters));
        }
    })
}
