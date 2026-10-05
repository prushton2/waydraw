use iced::{Border, Element, Theme};
use iced::border::Radius;
use iced::widget::{Column, MouseArea, button, column, container, image, pick_list, row, space, stack, text, text_input};
use iced::{Alignment::{self, Center}, Length::{self, Fill}};

use crate::window::window::Monitor;
use crate::window::{Message, Window, modules};

use crate::p2p::protocol::mouse_click::{MouseButton, MouseState};

pub enum UIState {
    Host{pin: String, key: String, wait: String, error: String},
    ConnectedHost,
    Client{pin_input: String, key_input: String, wait: String, error: String},
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
    Disconnect
}

pub fn view(this: &Window) -> iced::Element<'_, Message> {
    match &this.ui_state {
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

            return Widgets::ribbon_wrapper(this, column![
                text("Select a monitor").width(Fill).align_x(Center),
                iced::widget::Column::from_vec(monitor_buttons).width(Fill).align_x(Center),
                
                space().height(20),
                container(button("Allow Connections").on_press(Message::HostConnectFlow(modules::host::ConnectFlow::Register))).center_x(Fill),
                space().height(20),
                
                row![text("Pin"), space().width(24), text_input(pin, pin).on_input(|_| Message::None)],
                row![text("Key"), space().width(20), text_input(key, key).on_input(|_| Message::None)],
                
                text(wait).width(Fill).align_x(Center),
                text(error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
            ].into());
        },
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

            return Widgets::ribbon_wrapper(this, column![
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
                Column::from_vec(buttons.into()).width(Fill).align_x(Center),
                
                space().height(20),
                
                text(wait).width(Fill).align_x(Center),
                text(error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
            ].into());
        },
        UIState::Settings => {
            let mut known_hosts = vec![];

            for (key, host) in &this.config.known_hosts {
                known_hosts.push(
                    row![
                        text_input("Host", host).on_input(|v| Message::UpdateKnownHostName(key.clone(), v)),
                        text_input("Secret Key", key).on_input(|v| Message::UpdateKnownHostKey(key.clone(), v))
                    ]
                    .spacing(5)
                    .into()
                );
                known_hosts.push(space().height(5).into());
            }

            if known_hosts.len() == 0 {
                known_hosts = vec![text("No known hosts. Connect to a device and it will appear here.").into()];
            }

            return Widgets::ribbon_wrapper(this, column![
                    Widgets::center_separator(text("Select Theme").into(), Widgets::theme_dropdown(&this.config.theme)),
                    Widgets::center_separator(text("Edit known hosts").into(), Column::from_vec(known_hosts).width(Fill).into()),
                    Widgets::center_separator(text("Reset Device ID").into(), button("Reset").on_press(Message::ResetSecretKey).into())
                ].spacing(10).into()
            ).into();
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
                row![space().width(Fill), button("Disconnect").on_press(Message::Disconnect), space().width(Fill)]
            ].into()
        }
    }
}

struct Styles;

impl Styles {
    fn invisible_text_input(theme: &Theme) -> text_input::Style {
        text_input::Style {
            background: iced::Background::Color(theme.palette().background),
            border: Border { 
                color: theme.palette().background,
                width: 0.0,
                radius: Radius::new(0)
            },
            icon: theme.palette().background,
            placeholder: theme.palette().background,
            value: theme.palette().background,
            selection: theme.palette().background
        }
    }
    fn client_host_selected(theme: &Theme) -> button::Style {
        button::Style {
            background: None,
            text_color: theme.extended_palette().primary.base.color,
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
            text_color: theme.extended_palette().primary.base.text,
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
            text_color: theme.extended_palette().primary.base.text,
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
        button::Style  {
            background: Some(iced::Background::from(theme.palette().success)),
            text_color: theme.extended_palette().primary.base.text,
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
            text_color: theme.extended_palette().primary.base.text,
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
        UIUpdate::SetModeClient   => this.ui_state = UIState::Client { pin_input: String::from(""), key_input: String::from(""), wait: String::from(""), error: String::from("") },
        UIUpdate::SetModeHost     => this.ui_state = UIState::Host   { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") },
        UIUpdate::SetModeSettings => this.ui_state = UIState::Settings,
        UIUpdate::UpdateKeyTextbox(v) => {
            if let UIState::Client { pin_input: _, key_input, wait: _, error: _ } = &mut this.ui_state {
                *key_input = v
            } else if let UIState::Host { pin: _, key, wait: _, error: _ } = &mut this.ui_state {
                *key = v
            }
        },
        UIUpdate::UpdatePinTextbox(v) => {
            if let UIState::Client { pin_input, key_input: _, wait: _, error: _ } = &mut this.ui_state {
                *pin_input = v
            } else if let UIState::Host { pin, key: _, wait: _, error: _ } = &mut this.ui_state {
                *pin = v
            }
        },
        UIUpdate::UpdateWait(v) => {
            if let UIState::Client { pin_input: _, key_input: _, wait, error: _ } = &mut this.ui_state {
                *wait = v
            } else if let UIState::Host { pin: _, key: _, wait, error: _ } = &mut this.ui_state {
                *wait = v
            }
        },
        UIUpdate::UpdateError(v) => {
            if let UIState::Client { pin_input: _, key_input: _, wait: _, error } = &mut this.ui_state {
                *error = v
            } else if let UIState::Host { pin: _, key: _, wait: _, error } = &mut this.ui_state {
                *error = v
            }
        },
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

struct Widgets;

impl Widgets {
    pub fn theme_dropdown(theme: &iced::Theme) -> iced::Element<'static, Message> {
        let options = [
            iced::Theme::Light,
            iced::Theme::Dark,
            iced::Theme::Dracula,
            iced::Theme::Nord,
            iced::Theme::SolarizedLight,
            iced::Theme::SolarizedDark,
            iced::Theme::GruvboxLight,
            iced::Theme::GruvboxDark,
            iced::Theme::CatppuccinLatte,
            iced::Theme::CatppuccinFrappe,
            iced::Theme::CatppuccinMacchiato,
            iced::Theme::CatppuccinMocha,
            iced::Theme::TokyoNight,
            iced::Theme::TokyoNightStorm,
            iced::Theme::TokyoNightLight,
            iced::Theme::KanagawaWave,
            iced::Theme::KanagawaDragon,
            iced::Theme::KanagawaLotus,
            iced::Theme::Moonfly,
            iced::Theme::Nightfly,
            iced::Theme::Oxocarbon,
            iced::Theme::Ferra
        ];
    
        return pick_list(
            options,
            Some(theme.clone()), 
            Message::ChangeTheme
        ).into()
    }

    pub fn ribbon_wrapper<'a>(this: &'a Window, element: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
        let buttons: iced::Element<'static, Message> = match this.ui_state {
            UIState::Host { pin: _, key: _, wait: _, error: _ } => {
                row![
                    button("Host").width(Fill).style(move |t, _| {Styles::client_host_selected(t)}),
                    button("Client").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeClient)),
                    button("Settings").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeSettings)),
                ].into()
            },
            UIState::Client { pin_input: _, key_input: _, wait: _, error: _ } => {
                row![
                    button("Host").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeHost)),
                    button("Client").width(Fill).style(move |t, _| {Styles::client_host_selected(t)}),
                    button("Settings").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeSettings)),
                ].into()
            },
            UIState::Settings => {
                row![
                    button("Host").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeHost)),
                    button("Client").width(Fill).style(move |t, _| {Styles::client_host_unselected(t)}).on_press(Message::UIUpdate(UIUpdate::SetModeClient)),
                    button("Settings").width(Fill).style(move |t, _| {Styles::client_host_selected(t)}),
                ].into()
            }
            _ => row![].into()
        };

        return container(
            column![
                buttons,
                space().height(10),
                column![element]
                .max_width(400),
                space().height(Fill)
            ]
            .max_width(600)
            .align_x(Center)
        )
        .align_x(Center)
        .into()
    }

    pub fn center_separator<'a>(left: iced::Element<'a, Message>, right: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
        return row![
            row![space().width(Fill), text_input("", "").width(Length::Shrink).style(|t, _| Styles::invisible_text_input(t)), left].width(Length::Fill).align_y(Center),
            space().width(10),
            row![right, text_input("", "").width(Length::Shrink).style(|t, _| Styles::invisible_text_input(t))].width(Length::Fill).align_y(Center)
        ].into()
    }
}
