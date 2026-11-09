use std::sync::Arc;

use iced::{Alignment, Length, Task, widget::{button, column, container, row, space}};

use super::{Message, Window, Screen, screens, styles};

impl Window {
    pub fn boot() -> Self {
        let config = crate::Config::load_or_generate();
        Self {
            screen: Screen::Host(screens::Host::new()),
            config: config,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HostMessage(m) => {
                if let Screen::Host(host) = &mut self.screen {
                    host.update(m)
                } else {
                    Task::none()
                }
            },
            Message::ClientMessage(m) => {
                if let Screen::Client(client) = &mut self.screen {
                    client.update(m)
                } else {
                    Task::none()
                }
            },
            Message::SettingsMessage(m) => {
                if let Screen::Settings(settings) = &mut self.screen {
                    settings.update(&mut self.config, m)
                } else {
                    Task::none()
                }
            },

            Message::Tab(tab) => {
                self.screen = tab;
                Task::none()
            },
            Message::None => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        match &self.screen {
            Screen::Host(s) => {
                ribbon_wrapper(self, 
                    s.view()
                )
            }
            Screen::Client(s) => {
                ribbon_wrapper(self, 
                    s.view()
                )
            }
            Screen::Settings(s) => {
                ribbon_wrapper(self, 
                    s.view(&self.config)
                )
            }
            _ => {
                iced::widget::text("null").into()
            }
        }
    }
}

pub fn get_version() -> (u8, u8, u8) {
    // split off the -rc-n tag
    let version = env!("CARGO_PKG_VERSION")
        .split(".")
        .map(|s| match s.parse::<u8>() {
            Ok(t) => t,
            Err(_) => s.split_once('-').unwrap().0.parse::<u8>().unwrap(),
        })
        .collect::<Vec<u8>>();

    (version[0], version[1], version[2])
}


pub fn ribbon_wrapper<'a>(
    this: &'a Window,
    element: iced::Element<'a, Message>,
) -> iced::Element<'a, Message> {
    let buttons: iced::Element<'static, Message> = match this.screen {
        Screen::Host(_) => row![
            button("Host")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_selected(t) }),
            button("Client")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Client(screens::Client::new()))),
            button("Settings")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Settings(screens::Settings::new()))),
        ]
        .into(),
        Screen::Client(_) => row![
            button("Host")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Host(screens::Host::new()))),
            button("Client")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_selected(t) }),
            button("Settings")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Settings(screens::Settings::new()))),
        ]
        .into(),
        Screen::Settings(_) => row![
            button("Host")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Host(screens::Host::new()))),
            button("Client")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Screen::Client(screens::Client::new()))),
            button("Settings")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_selected(t) }),
        ]
        .into(),
        _ => row![].into(),
    };

    container(
        column![
            buttons,
            space().height(10),
            column![element].max_width(400),
            space().height(Length::Fill)
        ]
        .max_width(600)
        .align_x(Alignment::Center),
    )
    .align_x(Alignment::Center)
    .into()
}