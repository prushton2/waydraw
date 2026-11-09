use iced::border::Radius;
use iced::widget::{
    Column, MouseArea, button, column, container, image, pick_list, row, scrollable, space, stack,
    text, text_input,
};
use iced::{Alignment, Length};
use iced::{Border, Element, Padding, Theme};

use crate::iced_window::window::Monitor;
use crate::iced_window::{Message, Window, modules};

use crate::input::{MouseButton, MouseState};

pub enum UIState {
    Host {
        pin: String,
        key: String,
        wait: String,
        error: String,
    },
    ConnectedHost,
    Client {
        pin_input: String,
        key_input: String,
        wait: String,
        error: String,
    },
    ConnectedClient,
    Settings,
}

#[derive(Clone)]
pub enum UIUpdate {
    SetModeHost,
    SetModeClient,
    SetModeSettings,
    UpdatePinTextbox(String),
    UpdateKeyTextbox(String),
    UpdateWait(String),
    UpdateError(String),
    Connect,
    Disconnect,
}

pub fn view(this: &Window) -> iced::Element<'_, Message> {
    match &this.ui_state {
        UIState::Host {
            pin,
            key,
            wait,
            error,
        } => {
            let monitor_buttons: Vec<Element<'_, Message>> = this
                .monitors
                .iter()
                .map(|e: &Monitor| {
                    button(e.label.as_str())
                        .on_press(Message::SelectMonitor(e.id.clone()))
                        .style(|t, _| {
                            if this.selected_monitor == e.id {
                                Styles::selected_monitor(t)
                            } else {
                                Styles::unselected_monitor(t)
                            }
                        })
                        .into()
                })
                .collect();

            Widgets::ribbon_wrapper(
                this,
                column![
                    text("Select a monitor")
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    iced::widget::Column::from_vec(monitor_buttons)
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    space().height(20),
                    container(
                        button("Allow Connections").on_press(Message::HostConnectFlow(
                            modules::host::ConnectFlow::Register
                        ))
                    )
                    .center_x(Length::Fill),
                    space().height(20),
                    row![
                        text("Pin"),
                        space().width(24),
                        text_input(pin, pin).on_input(|_| Message::None)
                    ],
                    row![
                        text("Key"),
                        space().width(20),
                        text_input(key, key).on_input(|_| Message::None)
                    ],
                    text(wait).width(Length::Fill).align_x(Alignment::Center),
                    text(error)
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .style(|t| { text::danger(t) }),
                ]
                .into(),
            )
        }
        UIState::Client {
            pin_input,
            key_input,
            wait,
            error,
        } => {
            let mut buttons = vec![];

            for (key, host) in &this.config.known_hosts {
                buttons.push(
                    row![
                        button(host.as_str())
                            .on_press(Message::ClientConnectFlow(
                                modules::client::ConnectFlow::KeySubmitted(key.clone())
                            ))
                            .width(Length::Fill)
                            .style(|t, _| Styles::square_button(t)),
                        button("X")
                            .on_press(Message::RemoveKnownHost(key.clone()))
                            .style(|t, _| Styles::square_button(t)),
                    ]
                    .spacing(0)
                    .into(),
                );
                buttons.push(space().height(5).into())
            }

            Widgets::ribbon_wrapper(
                this,
                column![
                    text("Input device pin")
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    row![
                        text_input("000000", pin_input)
                            .on_input(|e| Message::UIUpdate(UIUpdate::UpdatePinTextbox(e))),
                        space().width(20),
                        button("Connect").on_press(Message::ClientConnectFlow(
                            modules::client::ConnectFlow::PinSubmitted(pin_input.clone())
                        ))
                    ],
                    text("OR").width(Length::Fill).align_x(Alignment::Center),
                    text("Input device key")
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    row![
                        text_input("", key_input)
                            .on_input(|e| Message::UIUpdate(UIUpdate::UpdateKeyTextbox(e))),
                        space().width(20),
                        button("Connect").on_press(Message::ClientConnectFlow(
                            modules::client::ConnectFlow::KeySubmitted(key_input.clone())
                        ))
                    ],
                    text("OR").width(Length::Fill).align_x(Alignment::Center),
                    text("Select previous device")
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    Column::from_vec(buttons)
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    space().height(20),
                    text(wait).width(Length::Fill).align_x(Alignment::Center),
                    text(error)
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .style(|t| { text::danger(t) }),
                ]
                .into(),
            )
        }
        UIState::Settings => {
            let mut known_hosts = vec![];

            for (key, host) in &this.config.known_hosts {
                known_hosts.push(
                    row![
                        text_input("Host", host)
                            .on_input(|v| Message::UpdateKnownHostName(key.clone(), v)),
                        text_input("Secret Key", key)
                            .on_input(|v| Message::UpdateKnownHostKey(key.clone(), v))
                    ]
                    .spacing(5)
                    .into(),
                );
                known_hosts.push(space().height(5).into());
            }

            if known_hosts.is_empty() {
                known_hosts = vec![
                    text("No known hosts. Connect to a device and it will appear here.").into(),
                ];
            }

            Widgets::ribbon_wrapper(
                this,
                scrollable(
                    column![
                        Widgets::center_title("Settings"),
                        Widgets::center_separator(
                            text("Select Theme").into(),
                            Widgets::theme_dropdown(&this.config.theme)
                        ),
                        Widgets::center_separator(
                            text("Edit known hosts").into(),
                            Column::from_vec(known_hosts).width(Length::Fill).into()
                        ),
                        Widgets::center_separator(
                            text("Reset Device ID").into(),
                            button("Reset").on_press(Message::ResetSecretKey).into()
                        ),
                        Widgets::center_title("Device Info"),
                        Widgets::center_separator(
                            text("Device").into(),
                            text(sysinfo::System::host_name().unwrap_or_default()).into()
                        ),
                        Widgets::center_separator(
                            text("OS").into(),
                            text(sysinfo::System::name().unwrap_or_default()).into()
                        ),
                        Widgets::center_separator(
                            text("Version").into(),
                            text(sysinfo::System::os_version().unwrap_or_default()).into()
                        ),
                        Widgets::center_separator(
                            text("Kernel").into(),
                            text(sysinfo::System::kernel_version().unwrap_or_default()).into()
                        ),
                    ]
                    .padding(Padding::ZERO.right(12))
                    .spacing(10),
                )
                .into(),
            )
        }
        UIState::ConnectedClient => container(stack![
            MouseArea::new(row![].width(Length::Fill).height(Length::Fill))
                .on_move(|point| { Message::MouseMove(point.x as i32, point.y as i32) })
                .on_press(Message::MouseClick(MouseButton::Left, MouseState::Pressed))
                .on_release(Message::MouseClick(MouseButton::Left, MouseState::Released))
                .on_right_press(Message::MouseClick(MouseButton::Right, MouseState::Pressed))
                .on_right_release(Message::MouseClick(
                    MouseButton::Right,
                    MouseState::Released
                )),
            match this.allocation.as_ref() {
                Some(allocation) => iced::Element::from(
                    image(allocation.handle())
                        .width(Length::Fill)
                        .height(Length::Fill)
                ),
                None => space().width(Length::Fill).height(Length::Fill).into(),
            }
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
        UIState::ConnectedHost => {
            fn info_row<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
                row![
                    text(label)
                        .width(Length::Fixed(80.0))
                        .style(text::secondary),
                    text(value),
                ]
                .spacing(12)
                .into()
            }

            let mut card = column![text("Connected").size(22)]
                .spacing(8)
                .align_x(Alignment::Start);

            if let Some(client_info) = this.client_info.as_ref() {
                card = card
                    .push(info_row("Device", deref(&client_info.host_name)))
                    .push(info_row("OS", deref(&client_info.name)))
                    .push(info_row("Version", deref(&client_info.os_version)))
                    .push(info_row("Kernel", deref(&client_info.kernel_version)));
            }

            card = card.push(
                container(
                    button(text("Disconnect").align_x(Alignment::Center))
                        .on_press(Message::Disconnect),
                )
                .center_x(Length::Fill)
                .padding(iced::Padding::ZERO.top(12)),
            );

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
}

// this is for PURE ui updates. This can (and should) be called from anywhere
pub fn update(this: &mut Window, message: UIUpdate) {
    match message {
        UIUpdate::SetModeClient => {
            this.ui_state = UIState::Client {
                pin_input: String::from(""),
                key_input: String::from(""),
                wait: String::from(""),
                error: String::from(""),
            }
        }
        UIUpdate::SetModeHost => {
            this.ui_state = UIState::Host {
                pin: String::from(""),
                key: String::from(""),
                wait: String::from(""),
                error: String::from(""),
            }
        }
        UIUpdate::SetModeSettings => this.ui_state = UIState::Settings,
        UIUpdate::UpdateKeyTextbox(v) => {
            if let UIState::Client {
                pin_input: _,
                key_input,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                *key_input = v
            } else if let UIState::Host {
                pin: _,
                key,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                *key = v
            }
        }
        UIUpdate::UpdatePinTextbox(v) => {
            if let UIState::Client {
                pin_input,
                key_input: _,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                *pin_input = v
            } else if let UIState::Host {
                pin,
                key: _,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                *pin = v
            }
        }
        UIUpdate::UpdateWait(v) => {
            if let UIState::Client {
                pin_input: _,
                key_input: _,
                wait,
                error: _,
            } = &mut this.ui_state
            {
                *wait = v
            } else if let UIState::Host {
                pin: _,
                key: _,
                wait,
                error: _,
            } = &mut this.ui_state
            {
                *wait = v
            }
        }
        UIUpdate::UpdateError(v) => {
            if let UIState::Client {
                pin_input: _,
                key_input: _,
                wait: _,
                error,
            } = &mut this.ui_state
            {
                *error = v
            } else if let UIState::Host {
                pin: _,
                key: _,
                wait: _,
                error,
            } = &mut this.ui_state
            {
                *error = v
            }
        }
        UIUpdate::Connect => {
            if let UIState::Client {
                pin_input: _,
                key_input: _,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                this.ui_state = UIState::ConnectedClient;
            } else if let UIState::Host {
                pin: _,
                key: _,
                wait: _,
                error: _,
            } = &mut this.ui_state
            {
                this.ui_state = UIState::ConnectedHost;
            }
        }
        UIUpdate::Disconnect => {
            if let UIState::ConnectedClient = &mut this.ui_state {
                this.ui_state = UIState::Client {
                    pin_input: String::from(""),
                    key_input: String::from(""),
                    wait: String::from(""),
                    error: String::from(""),
                };
            } else if let UIState::ConnectedHost = &mut this.ui_state {
                this.ui_state = UIState::Host {
                    pin: String::from(""),
                    key: String::from(""),
                    wait: String::from(""),
                    error: String::from(""),
                };
            }
        }
    }
}

struct Widgets;

impl Widgets {
   

    pub fn ribbon_wrapper<'a>(
        this: &'a Window,
        element: iced::Element<'a, Message>,
    ) -> iced::Element<'a, Message> {
        let buttons: iced::Element<'static, Message> = match this.ui_state {
            UIState::Host {
                pin: _,
                key: _,
                wait: _,
                error: _,
            } => row![
                button("Host")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_selected(t) }),
                button("Client")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeClient)),
                button("Settings")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeSettings)),
            ]
            .into(),
            UIState::Client {
                pin_input: _,
                key_input: _,
                wait: _,
                error: _,
            } => row![
                button("Host")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeHost)),
                button("Client")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_selected(t) }),
                button("Settings")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeSettings)),
            ]
            .into(),
            UIState::Settings => row![
                button("Host")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeHost)),
                button("Client")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_unselected(t) })
                    .on_press(Message::UIUpdate(UIUpdate::SetModeClient)),
                button("Settings")
                    .width(Length::Fill)
                    .style(move |t, _| { Styles::client_host_selected(t) }),
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

    pub fn center_separator<'a>(
        left: iced::Element<'a, Message>,
        right: iced::Element<'a, Message>,
    ) -> iced::Element<'a, Message> {
        row![
            row![
                space().width(Length::Fill),
                text_input("", "")
                    .width(Length::Shrink)
                    .style(|t, _| Styles::invisible_text_input(t)),
                left
            ]
            .width(Length::Fill)
            .align_y(Alignment::Center),
            space().width(10),
            row![
                right,
                text_input("", "")
                    .width(Length::Shrink)
                    .style(|t, _| Styles::invisible_text_input(t))
            ]
            .width(Length::Fill)
            .align_y(Alignment::Center)
        ]
        .into()
    }

    pub fn center_title<'a>(inner_text: &'a str) -> iced::Element<'a, Message> {
        column![
            text(""),
            row![
                space().width(Length::Fill),
                text(inner_text).size(22).style(text::primary),
                space().width(Length::Fill),
            ]
        ]
        .into()
    }
}

fn deref(option: &Option<String>) -> &str {
    option.as_deref().unwrap_or("Unknown")
}
