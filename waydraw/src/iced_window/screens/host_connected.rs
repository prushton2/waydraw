use std::sync::{Arc, RwLock};

use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Alignment, Length, Task};

use super::host_worker;
use crate::iced_window::screens::host::Monitor;
use crate::iced_window::{Message, Screen, styles};
use crate::p2p::protocol; // for out.send(..).await and cmd_rx.next().await

#[derive(Clone, Debug)]
pub enum HostConnectedMessage {
    WorkerEvent(host_worker::OutgoingWorkerEvent),
    Disconnect(String),
    AnswerConfirmConnectionPrompt(bool),
}

pub struct HostConnected {
    pub key: String,
    pub pin: String,
    pub error: String,
    pub sender: Option<Sender<host_worker::IncomingWorkerEvent>>,
    pub client_info: Option<Arc<RwLock<protocol::ClientHello>>>,
    pub static_client_info: Option<protocol::ClientHello>,
    pub monitor: Option<Monitor>,
    pub confirm_connect_prompt: Option<Sender<bool>>,
}

impl HostConnected {
    pub fn new(monitor: Option<Monitor>) -> Self {
        Self {
            key: String::from(""),
            pin: String::from(""),
            error: if monitor.is_none() {
                String::from("No monitor selected")
            } else {
                String::from("")
            },
            sender: None,
            client_info: None,
            static_client_info: None,
            monitor,
            confirm_connect_prompt: None,
        }
    }

    pub fn update(&mut self, message: HostConnectedMessage) -> Task<Message> {
        match message {
            HostConnectedMessage::WorkerEvent(event) => {
                match event {
                    host_worker::OutgoingWorkerEvent::UpdateKnownHosts(hostname, key) => {
                        return Task::done(Message::UpdateKnownHosts(hostname, key));
                    }
                    host_worker::OutgoingWorkerEvent::Disconnect(reason) => {
                        return Task::done(Message::HostConnectedMessage(
                            HostConnectedMessage::Disconnect(reason),
                        ));
                    }
                    host_worker::OutgoingWorkerEvent::UpdateKey(key) => self.key = key,
                    host_worker::OutgoingWorkerEvent::UpdatePin(pin) => self.pin = pin,
                    host_worker::OutgoingWorkerEvent::Failed(e) => self.error = e,
                    host_worker::OutgoingWorkerEvent::SetStaticClientInfo(t) => {
                        self.static_client_info = Some(t);
                    }
                    host_worker::OutgoingWorkerEvent::SetClientInfo(t) => {
                        self.client_info = Some(t);
                    }
                    host_worker::OutgoingWorkerEvent::Ready(t) => self.sender = Some(t),
                    host_worker::OutgoingWorkerEvent::UpdateClientWindowSize(w, h) => {
                        if let Some(client_info) = &self.client_info {
                            let mut lock = client_info.write().unwrap();
                            lock.window_width = w;
                            lock.window_height = h;
                        }
                    }
                    host_worker::OutgoingWorkerEvent::SetConfirmConnectPromptSender(sender) => {
                        self.confirm_connect_prompt = sender;
                    }
                }
                Task::none()
            }
            HostConnectedMessage::Disconnect(reason) => {
                if let Some(mut sender) = self.sender.clone() {
                    Task::perform(
                        async move {
                            let _ = sender
                                .send(host_worker::IncomingWorkerEvent::Disconnect(reason))
                                .await;
                        },
                        |_| Message::Tab(Arc::new(|| Screen::Host(super::Host::new()))),
                    )
                } else {
                    Task::done(Message::Tab(Arc::new(|| Screen::Host(super::Host::new()))))
                }
            }
            HostConnectedMessage::AnswerConfirmConnectionPrompt(value) => {
                if let Some(mut sender) = self.confirm_connect_prompt.clone() {
                    let _ = sender.try_send(value);
                    sender.disconnect();
                }
                self.confirm_connect_prompt = None;
                if !value {
                    Task::done(Message::HostConnectedMessage(
                        HostConnectedMessage::Disconnect("Host refused connection".to_owned()),
                    ))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        fn info_row<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Message> {
            row![
                text(label)
                    .width(Length::Fixed(120.0))
                    .style(text::secondary),
                text(value),
            ]
            .spacing(12)
            .into()
        }

        fn textbox_row<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Message> {
            row![
                text(label)
                    .width(Length::Fixed(80.0))
                    .style(text::secondary),
                text_input("", value).style(|t, _| styles::context_textbox(t)),
            ]
            .spacing(12)
            .into()
        }

        let mut card = column![].spacing(8).align_x(Alignment::Start);

        if self.confirm_connect_prompt.is_some()
            && let Some(info) = self.static_client_info.as_ref()
        {
            card = card
                .push(text("Confirm Connection").size(22))
                .push(text(
                    "A waydraw client is attempting to connect to your device",
                ))
                .push(info_row("Device Name", deref(&info.host_name)))
                .push(info_row("OS", deref(&info.name)))
                .push(info_row("OS Version", deref(&info.os_version)))
                .push(info_row("Kernel Version", deref(&info.kernel_version)))
                .push(container(row![
                    space().width(Length::Fill),
                    button("Accept").on_press(Message::HostConnectedMessage(
                        HostConnectedMessage::AnswerConfirmConnectionPrompt(true)
                    ))
                    .style(button::primary),
                    space().width(Length::Fixed(32.0)),
                    button("Reject").on_press(Message::HostConnectedMessage(
                        HostConnectedMessage::AnswerConfirmConnectionPrompt(false)
                    ))
                    .style(button::secondary),
                    space().width(Length::Fill),
                ]));
        } else if !self.error.is_empty() {
            card = card
                .push(text("Error Connecting").size(22))
                .push(text(&self.error).style(text::danger));
        } else if let Some(static_client_info) = self.static_client_info.as_ref() {
            card = card
                .push(text("Connected").size(22))
                .push(info_row("Device", deref(&static_client_info.host_name)))
                .push(info_row("OS", deref(&static_client_info.name)))
                .push(info_row("Version", deref(&static_client_info.os_version)))
                .push(info_row(
                    "Kernel",
                    deref(&static_client_info.kernel_version),
                ))
                .push(
                    container(
                        button(
                            text(if self.error.is_empty() {
                                "Disconnect"
                            } else {
                                "Retry"
                            })
                            .align_x(Alignment::Center),
                        )
                        .on_press(Message::HostConnectedMessage(
                            HostConnectedMessage::Disconnect(
                                "Host initiated disconnect".to_string(),
                            ),
                        )),
                    )
                    .center_x(Length::Fill)
                    .padding(iced::Padding::ZERO.top(12)),
                );
        } else if self.key.is_empty() {
            card = card
                .push(text("Registering...").size(22))
                .push(textbox_row("Pin", &self.pin))
                .push(textbox_row("Key", &self.key))
                .push(text(&self.error).style(text::danger));
        } else {
            card = card
                .push(text("Awaiting Connection").size(22))
                .push(textbox_row("Pin", &self.pin))
                .push(textbox_row("Key", &self.key))
                .push(text(&self.error).style(text::danger));
        }

        container(
            container(card)
                .padding(20)
                .max_width(420)
                .style(container::rounded_box),
        )
        .center(Length::Fill)
        .into()
    }
}

fn deref(option: &Option<String>) -> &str {
    option.as_deref().unwrap_or("Unknown")
}
