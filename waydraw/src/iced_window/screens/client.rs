use std::sync::Arc;

use iced::{
    Alignment, Length, Task,
    widget::{Column, button, column, row, space, text, text_input},
};

use crate::iced_window::{Message, Screen, screens};

pub struct Client {
    pin_input: String,
    key_input: String,
    wait: String,
    err: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            pin_input: String::from(""),
            key_input: String::from(""),
            wait: String::from(""),
            err: String::from(""),
        }
    }

    pub fn update(&mut self, message: ClientMessage) -> Task<Message> {
        match message {
            ClientMessage::UpdateKey(s) => {
                self.key_input = s;
                Task::none()
            }
            ClientMessage::UpdatePin(s) => {
                self.pin_input = s;
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self, config: &'a crate::Config) -> iced::Element<'a, Message> {
        let mut buttons: Vec<iced::Element<'a, Message>> = vec![];

        for (key, host) in &config.known_hosts {
            buttons.push(
                row![
                    button(host.as_str())
                        .on_press(Message::Tab(Arc::new({
                            let key = key.clone();
                            move || {
                                Screen::ClientConnected(screens::ClientConnected::from_key(
                                    key.clone(),
                                ))
                            }
                        })))
                        .width(Length::Fill)
                        .style(|t, _| crate::iced_window::styles::square_button(t)),
                    // button("X")
                    //     .on_press(Message::RemoveKnownHost(key.clone()))
                    //     .style(|t, _| styles::square_button(t)),
                ]
                .spacing(0)
                .into(),
            );
            buttons.push(space().height(5).into())
        }

        column![
            text("Input device pin")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            row![
                text_input("000000", &self.pin_input)
                    .on_input(|e| Message::ClientMessage(ClientMessage::UpdatePin(e))),
                space().width(20),
                button("Connect").on_press(Message::Tab(Arc::new({
                    let pin_ref = self.pin_input.clone();
                    move || {
                        Screen::ClientConnected(screens::ClientConnected::from_pin(pin_ref.clone()))
                    }
                })))
            ],
            text("OR").width(Length::Fill).align_x(Alignment::Center),
            text("Input device key")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            row![
                text_input("", &self.key_input)
                    .on_input(|e| Message::ClientMessage(ClientMessage::UpdateKey(e))),
                space().width(20),
                button("Connect").on_press(Message::Tab(Arc::new({
                    let key_ref = self.key_input.clone();
                    move || {
                        Screen::ClientConnected(screens::ClientConnected::from_key(key_ref.clone()))
                    }
                })))
            ],
            text("OR").width(Length::Fill).align_x(Alignment::Center),
            text("Select previous device")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            Column::from_vec(buttons)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            space().height(20),
            text(&self.wait)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            text(&self.err)
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .style(|t| { text::danger(t) }),
        ]
        .into()
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    UpdatePin(String),
    UpdateKey(String),
}
