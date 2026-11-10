use iced::{Task, Length, Alignment, widget::{row, space, column, text, text_input, button}};

use crate::iced_window::Message;

#[derive(Clone)]
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
            err: String::from("")
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

    pub fn view(&self) -> iced::Element<'_, Message> {
        // let mut buttons = vec![];

        // for (key, host) in &self.config.known_hosts {
        //     buttons.push(
        //         row![
        //             button(host.as_str())
        //                 .on_press(Message::ClientConnectFlow(
        //                     modules::client::ConnectFlow::KeySubmitted(key.clone())
        //                 ))
        //                 .width(Length::Fill)
        //                 .style(|t, _| Styles::square_button(t)),
        //             button("X")
        //                 .on_press(Message::RemoveKnownHost(key.clone()))
        //                 .style(|t, _| Styles::square_button(t)),
        //         ]
        //         .spacing(0)
        //         .into(),
        //     );
        //     buttons.push(space().height(5).into())
        // }

        column![
            text("Input device pin")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            row![
                text_input("000000", &self.pin_input)
                    .on_input(|e| Message::ClientMessage(ClientMessage::UpdatePin(e))),
                space().width(20),
                button("Connect").on_press(Message::Tab(crate::iced_window::Screen::ClientConnected(super::ClientConnected::new(self.pin_input.clone(), String::from("")))))
            ],
            text("OR").width(Length::Fill).align_x(Alignment::Center),
            text("Input device key")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            row![
                text_input("", &self.key_input)
                    .on_input(|e| Message::ClientMessage(ClientMessage::UpdateKey(e))),
                space().width(20),
                button("Connect").on_press(Message::Tab(crate::iced_window::Screen::ClientConnected(super::ClientConnected::new(String::from(""), self.key_input.clone()))))
            ],
            text("OR").width(Length::Fill).align_x(Alignment::Center),
            text("Select previous device")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            // Column::from_vec(buttons)
            //     .width(Length::Fill)
            //     .align_x(Alignment::Center),
            space().height(20),
            text(&self.wait).width(Length::Fill).align_x(Alignment::Center),
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
