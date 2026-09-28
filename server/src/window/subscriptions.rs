use iced::Subscription;

use super::{Window, Message};
use crate::window::modules::receive_stream;
use crate::window::modules::screencap_stream;

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions = vec![];

    if window.connected {
        subscriptions.push(
            iced::Subscription::run_with(receive_stream::P2PObject(window.p2p.clone()), receive_stream::p2p_stream),
        );
    }
    
    if window.connected && window.recording.read().unwrap().is_some() {
        let params = screencap_stream::ScreencapStreamParameters {
            recording: window.recording.clone(),
            encoder: window.encoder.clone(),
            p2p: window.p2p.clone(),
            client_window_size: window.client_window_size.clone()
        };

        subscriptions.push(
            iced::Subscription::run_with(params, screencap_stream::screencap_stream)
        );
    };

    return iced::Subscription::batch(subscriptions);
}

