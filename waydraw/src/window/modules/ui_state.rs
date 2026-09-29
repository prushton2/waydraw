use iced::{Border, Element, Theme};
use iced::border::Radius;
use iced::widget::{button, column, container, row, space, stack, text, text_input, MouseArea, image};
use iced::{Alignment::Center, Length::Fill};

use crate::window::window::Monitor;
use crate::window::{Message, Window, modules};

use crate::p2p::protocol::mouse_click::{MouseButton, MouseState};

pub enum UIState {
    Host{pin: String, key: String, wait: String, error: String},
    ConnectedHost,
    Client{pin_input: String, key_input: String, wait: String, error: String},
    ConnectedClient,
}

#[derive(Clone)]
pub enum UIUpdate {
    SetModeHost,
    SetModeClient,
    UpdatePinTextbox(String),
    UpdateKeyTextbox(String),
    // UpdateWait(String),
    // UpdateError(String),
    Connect,
    Disconnect
}

pub fn view(this: &Window) -> iced::Element<'_, Message> {
    match &this.ui_state {
        UIState::Client{pin_input, key_input, wait, error} => {
            let mut buttons = vec![];

            for (key, host) in &this.config.known_hosts {
                buttons.push(
                    row![
                        button(host.as_str()).on_press(Message::ClientConnectFlow(modules::client::ConnectFlow::KeySubmitted(key.clone()))).width(Fill).style(|t, _| Styles::square_button(t)),
                        button("X").on_press(Message::RemoveKnownHost(key.clone())).style(|t, _| Styles::square_button(t)),
                    ]
                    .spacing(0)
                    .into()
                );
                buttons.push(space().height(5).into())
            }

            return container (
                column![
                    row![
                        button("Host").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeHost)),
                        button("Client").width(Fill).style(move |t, _| {Styles::client_host_selected(t)}),
                    ],
                    text("Input device pin").width(Fill).align_x(Center),
                    row![
                        text_input("000000", pin_input).on_input(|e| Message::UIUpdate(UIUpdate::UpdatePinTextbox(e))),
                        space().width(20),
                        button("Connect").on_press(Message::ClientConnectFlow(modules::client::ConnectFlow::PinSubmitted(pin_input.clone())))
                    ],
                    
                    text("OR").width(Fill).align_x(Center),
                    
                    text("Input device key").width(Fill).align_x(Center),
                    row![
                        text_input("", key_input).on_input(|e| Message::UIUpdate(UIUpdate::UpdateKeyTextbox(e))),
                        space().width(20),
                        button("Connect").on_press(Message::ClientConnectFlow(modules::client::ConnectFlow::KeySubmitted(key_input.clone())))
                    ],
                    
                    text("OR").width(Fill).align_x(Center),
                    
                    text("Select previous device").width(Fill).align_x(Center),
                    iced::widget::Column::from_vec(buttons.into()).width(Fill).align_x(Center),
                    
                    space().height(20),
                    
                    text(wait).width(Fill).align_x(Center),
                    text(error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
                ]
                .max_width(400)
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        },
        UIState::Host{pin, key, wait, error} => {

            let monitor_buttons: Vec<Element<'_, Message>> = this.monitors
                .iter()
                .map(|e: &Monitor| 
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
                ).collect();

            return container (
                column![
                    row![
                        button("Host").width(Fill).style(move |t, _| {Styles::client_host_selected(t)}),
                        button("Client").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeClient)),
                    ],
                    column![
                        text("Select a monitor").width(Fill).align_x(Center),
                        iced::widget::Column::from_vec(monitor_buttons).width(Fill).align_x(Center),
                        
                        space().height(20),
                        container(button("Allow Connections").on_press(Message::HostConnectFlow(modules::host::ConnectFlow::Register))).center_x(Fill),
                        space().height(20),
                        
                        row![text("Pin"), space().width(24), text_input(pin, pin).on_input(|_| Message::None)],
                        row![text("Key"), space().width(20), text_input(key, key).on_input(|_| Message::None)],
                        
                        text(wait).width(Fill).align_x(Center),
                        text(error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
                    ]
                ]
                .max_width(400)
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        },
        UIState::ConnectedClient => {
            return container (
            stack![
                MouseArea::new(
                    row![]
                    .width(Fill)
                    .height(Fill)
                )
                .on_move(|point| {return Message::MouseMove(point.x as i32, point.y as i32)})
    
                .on_press        (Message::MouseClick(MouseButton::Left,  MouseState::Pressed ))
                .on_release      (Message::MouseClick(MouseButton::Left,  MouseState::Released))
                .on_right_press  (Message::MouseClick(MouseButton::Right, MouseState::Pressed ))
                .on_right_release(Message::MouseClick(MouseButton::Right, MouseState::Released)),

                match this.allocation.as_ref() {
                    Some(allocation) => iced::Element::from(
                        image(allocation.handle())
                            .width(Fill)
                            .height(Fill)
                    ),
                    None => space().width(Fill).height(Fill).into()
                }
            ]

        )
        .width(Fill)
        .height(Fill)
        .into()
        },
        UIState::ConnectedHost => {
            return column![
                button("Disconnect").on_press(Message::Disconnect)
            ].into()
        }
    }
}

struct Styles;

impl Styles {
    fn client_host_selected(theme: &Theme) -> button::Style {
        button::Style {
            background: None,
            text_color: theme.palette().primary,
            border: Border {
                color: theme.palette().background,
                width: 0.0,
                radius: Radius::new(0)
            },
            shadow: iced::Shadow {
                color: theme.palette().primary,
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 0.0
            },
            snap: false
        }
    }

    fn client_host_unselected(theme: &Theme) -> button::Style {
        button::Style {
            background: Some(iced::Background::from(theme.palette().primary)),
            text_color: theme.palette().background,
            border: Border {
                color: iced::Color::from_rgb(0.0, 0.0, 0.0),
                width: 0.0,
                radius: Radius::new(0)
            },
            shadow: iced::Shadow {
                color: theme.palette().primary,
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 0.0
            },
            snap: false
        }
    }

    fn square_button(theme: &Theme) -> button::Style {
        button::Style {
            background: Some(iced::Background::from(theme.palette().primary)),
            text_color: theme.palette().background,
            border: Border {
                color: iced::Color::from_rgb(0.0, 0.0, 0.0),
                width: 0.0,
                radius: Radius::new(0)
            },
            shadow: iced::Shadow {
                color: theme.palette().background,
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 0.0
            },
            snap: false
        }
    }

    fn selected_monitor(theme: &Theme) -> button::Style {
        button::Style {
            background: Some(iced::Background::from(theme.palette().success)),
            text_color: theme.palette().background,
            border: Border {
                color: iced::Color::from_rgb(0.0, 0.0, 0.0),
                width: 0.0,
                radius: Radius::new(2)
            },
            shadow: iced::Shadow {
                color: theme.palette().background,
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 0.0
            },
            snap: false
        }
    }

    fn unselected_monitor(theme: &Theme) -> button::Style {
        button::Style {
            background: Some(iced::Background::from(theme.palette().primary)),
            text_color: theme.palette().background,
            border: Border {
                color: iced::Color::from_rgb(0.0, 0.0, 0.0),
                width: 0.0,
                radius: Radius::new(2)
            },
            shadow: iced::Shadow {
                color: theme.palette().background,
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 0.0
            },
            snap: false
        }
    }
}

// this is for PURE ui updates. This can (and should) be called from anywhere 
pub fn update(this: &mut Window, message: UIUpdate) {
    match message {
        UIUpdate::SetModeClient => this.ui_state = UIState::Client { pin_input: String::from(""), key_input: String::from(""), wait: String::from(""), error: String::from("") },
        UIUpdate::SetModeHost   => this.ui_state = UIState::Host   { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") },
        UIUpdate::UpdateKeyTextbox(v) => {
            if let UIState::Client { pin_input: _, key_input, wait: _, error: _ } = &mut this.ui_state {
                *key_input = v
            }
        },
        UIUpdate::UpdatePinTextbox(v) => {
            if let UIState::Client { pin_input, key_input: _, wait: _, error: _ } = &mut this.ui_state {
                *pin_input = v
            }
        },
        // UIUpdate::UpdateWait(v) => {
        //     if let UIState::Client { pin_input: _, key_input: _, wait, error: _ } = &mut this.ui_state {
        //         *wait = v
        //     } else if let UIState::Host { pin: _, key: _, wait, error: _ } = &mut this.ui_state {
        //         *wait = v
        //     }
        // },
        // UIUpdate::UpdateError(v) => {
        //     if let UIState::Client { pin_input: _, key_input: _, wait: _, error } = &mut this.ui_state {
        //         *error = v
        //     } else if let UIState::Host { pin: _, key: _, wait: _, error } = &mut this.ui_state {
        //         *error = v
        //     }
        // },
        UIUpdate::Connect => {
            if let UIState::Client { pin_input: _, key_input: _, wait: _, error: _ } = &mut this.ui_state {
                this.ui_state = UIState::ConnectedClient;
            } else if let UIState::Host { pin: _, key: _, wait: _, error: _ } = &mut this.ui_state {
                this.ui_state = UIState::ConnectedHost;
            }
        },
        UIUpdate::Disconnect => {
            if let UIState::ConnectedClient = &mut this.ui_state {
                this.ui_state = UIState::Client { pin_input: String::from(""), key_input: String::from(""), wait: String::from(""), error: String::from("") };
            } else if let UIState::ConnectedHost = &mut this.ui_state {
                this.ui_state = UIState::Host { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") };
            }
        }
    }
}