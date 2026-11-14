use std::{collections::hash_map, sync::Arc};

use iced::{
    Alignment, Length, Subscription, Task,
    widget::{button, column, container, row, space},
};

use crate::iced_window::{
    screen_capture,
    screens::{client_connected, host_connected},
};

use super::{Message, Screen, Window, screens, styles};

impl Window {
    pub fn boot() -> Self {
        let config = crate::Config::load_or_generate();
        Self {
            window_size: (100, 100),
            screen: Screen::Host(screens::Host::new()),
            config,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HostConnectedMessage(m) => {
                if let Screen::HostConnected(host) = &mut self.screen {
                    host.update(m)
                } else {
                    Task::none()
                }
            }
            Message::ClientConnectedMessage(m) => {
                if let Screen::ClientConnected(host) = &mut self.screen {
                    host.update(m, self.window_size)
                } else {
                    Task::none()
                }
            }
            Message::HostMessage(m) => {
                if let Screen::Host(host) = &mut self.screen {
                    host.update(m)
                } else {
                    Task::none()
                }
            }
            Message::ClientMessage(m) => {
                if let Screen::Client(client) = &mut self.screen {
                    client.update(m)
                } else {
                    Task::none()
                }
            }
            Message::SettingsMessage(m) => {
                if let Screen::Settings(settings) = &mut self.screen {
                    settings.update(&mut self.config, m)
                } else {
                    Task::none()
                }
            }

            Message::WindowResize(w, h) => {
                self.window_size = (w, h);
                
                if let Screen::ClientConnected(client) = &mut self.screen {
                    client.update(client_connected::ClientConnectedMessage::WindowResize(w, h), (w, h))
                } else {
                    Task::none()
                }
            }
            Message::Tab(tab) => {
                self.screen = tab.as_ref()();
                Task::none()
            }
            Message::UpdateKnownHosts(hostname, key) => {
                // Only insert if it doesnt exist to preserve the edited hostname if the client decides to edit it
                if let hash_map::Entry::Vacant(e) = self.config.known_hosts.entry(key) {
                    e.insert(hostname);
                    self.config.write();
                }
                Task::none()
            }
            Message::RemoveKnownHost(key) => {
                let _ = self.config.known_hosts.remove(&key);
                Task::none()
            }
            Message::None => Task::none(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        match &self.screen {
            Screen::HostConnected(s) => s.view(),
            Screen::ClientConnected(s) => s.view(),
            Screen::Host(s) => ribbon_wrapper(self, s.view()),
            Screen::Client(s) => ribbon_wrapper(self, s.view(&self.config)),
            Screen::Settings(s) => ribbon_wrapper(self, s.view(&self.config)),
        }
    }
}

pub fn subscription(window: &Window) -> Subscription<Message> {
    let mut subscriptions: Vec<Subscription<Message>> = match &window.screen {
        Screen::HostConnected(host) => {
            let props = host_connected::Params {
                id: 0,
                secret_key: window.config.secret_key.clone(),
                monitor: host.monitor.clone(),
            };
            let mut s = vec![
                iced::Subscription::run_with(props, host_connected::worker).map(|e| {
                    Message::HostConnectedMessage(
                        host_connected::HostConnectedMessage::WorkerEvent(e),
                    )
                }),
            ];

            if let Some(sender) = &host.sender {
                let params = screen_capture::Params {
                    monitor: host.monitor.clone(),
                    sender: sender.clone(),
                };

                s.push(iced::Subscription::run_with(params, screen_capture::worker));
            };

            s
        }
        Screen::ClientConnected(t) => {
            let props = client_connected::Params {
                id: 0,
                pin: t.pin.clone(),
                key: t.key.clone(),
                window_size: window.window_size,
            };

            vec![
                iced::Subscription::run_with(props, client_connected::worker).map(|e| {
                    Message::ClientConnectedMessage(
                        client_connected::ClientConnectedMessage::WorkerEvent(e),
                    )
                }),
                iced::keyboard::listen().map(|e| {
                    Message::ClientConnectedMessage(
                        client_connected::ClientConnectedMessage::KeyEvent(e),
                    )
                }),
            ]
        }
        _ => vec![],
    };

    subscriptions.push(
        iced::window::resize_events()
            .map(|e| Message::WindowResize(e.1.width as usize, e.1.height as usize)),
    );

    iced::Subscription::batch(subscriptions)
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
                .on_press(Message::Tab(Arc::new(|| Screen::Client(
                    screens::Client::new()
                )))),
            button("Settings")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Arc::new(|| Screen::Settings(
                    screens::Settings::new()
                )))),
        ]
        .into(),
        Screen::Client(_) => row![
            button("Host")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Arc::new(
                    || Screen::Host(screens::Host::new())
                ))),
            button("Client")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_selected(t) }),
            button("Settings")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Arc::new(|| Screen::Settings(
                    screens::Settings::new()
                )))),
        ]
        .into(),
        Screen::Settings(_) => row![
            button("Host")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Arc::new(
                    || Screen::Host(screens::Host::new())
                ))),
            button("Client")
                .width(Length::Fill)
                .style(move |t, _| { styles::client_host_unselected(t) })
                .on_press(Message::Tab(Arc::new(|| Screen::Client(
                    screens::Client::new()
                )))),
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
