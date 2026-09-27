use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use iced::Length::Fill;
use iced::Task;
use iced::alignment::Horizontal::Center;
use iced::widget::{space, button, column, container, row, text, text_input};

use crate::mouse;
use crate::window::modules::{connect_flow, receive_stream};

use super::{Window, Message, Monitor};

impl Window {
    pub fn boot() -> Self {
        let mut error = String::from("");

        let (monitors, monitor_labels) = match Self::read_monitors() {
            Ok(t) => t,
            Err(e) => {
                error = format!("Error reading monitors: {}\n\n", e);
                (vec![], vec![])
            }
        };
        
        let mut mouse: Box<dyn mouse::Mouse> = Box::new(mouse::DummyMouse::new());
        if !cfg!(debug_assertions) {
            mouse = Box::new(mouse::EnigoMouse::new());
        }

        let this = Self {
            p2p: Arc::new(RwLock::new(None)),
            mouse: mouse,
            h264_instance: Arc::new(Mutex::new(openh264::encoder::Encoder::new().unwrap())),

            client_window_size: Arc::new(std::sync::Mutex::new((100, 100))),
            connected: false,

            available_monitors: monitors,
            selected_monitor: None,
            recording: Arc::new(std::sync::RwLock::new(None)),

            pin: None,
            key: None,

            wait_reason: String::from(""),
            error: error,

            labels: monitor_labels,
        };

        this
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::ConnectFlow(sub_message) => {
                connect_flow::update(self, sub_message).map(Message::ConnectFlow)
            },

            Message::P2PMessage(m) => {
                receive_stream::update(self, m).map(Message::P2PMessage)
            },

            Message::SelectMonitor(i) => {
                self.selected_monitor = Some(i);
                Task::none()
            },

            Message::None => {
                Task::none()
            },

            Message::Null(_) => {
                Task::none()
            },

        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.connected {
            return button("Disconnect").on_press(Message::ConnectFlow(connect_flow::ConnectFlow::Disconnect)).into();
        }

        let pin = self.pin.clone().unwrap_or("".to_owned());
        let key = self.key.clone().unwrap_or("".to_owned());

        let mut buttons: Vec<iced::Element<'_, Message>> = vec![];

        for i in 0..self.labels.len() {
            let mut button = button(self.labels[i].as_str()).on_press(Message::SelectMonitor(i)).width(Fill);

            if Some(i) == self.selected_monitor {
                button = button.style(|theme, status| {
                    button::subtle(theme, status)
                });
            }

            buttons.push(
                button.into()
            );
            buttons.push(space().height(5).into())
        }

        container (
            column![
                text("Select a monitor").width(Fill).align_x(Center),
                iced::widget::Column::from_vec(buttons).width(Fill).align_x(Center),
                
                space().height(20),
                container(button("Allow Connections").on_press(Message::ConnectFlow(connect_flow::ConnectFlow::Register))).center_x(Fill),
                space().height(20),
                
                row![text("Pin"), space().width(24), text_input(&pin, &pin).on_input(|_| Message::None)],
                row![text("Key"), space().width(20), text_input(&key, &key).on_input(|_| Message::None)],
                
                text(&self.wait_reason).width(Fill).align_x(Center),
                text(&self.error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
            ]
            .max_width(400)

        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    fn read_monitors() -> Result<(Vec<Monitor>, Vec<String>), String> {
        let positions = display_info::DisplayInfo::all().unwrap_or_default();

        let monitors = pinray::enumerate_sources()
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|e| {
                match e {
                    pinray::CaptureSource::Display(display) => Some(display),
                    _ => None
                }
            })
            .map(|pinray_source| {
                // pinray ids are `display:<name>`, where <name> is the same device name display_info reports (`DP-1`, `\\.\DISPLAY1`, ...).
                let name = pinray_source.id.0.strip_prefix("display:").unwrap_or(&pinray_source.id.0);

                let displayinfo_source = positions
                    .iter()
                    .find(|e| e.name == name)
                    .map(|e| e)
                    .unwrap();

                Monitor {
                    id: pinray_source.id.0.clone(),
                    name: pinray_source.name,
                    position: (displayinfo_source.x, displayinfo_source.y),
                    resolution: (pinray_source.width, pinray_source.height),
                    scale: displayinfo_source.scale_factor
                }
            })
            .collect::<Vec<Monitor>>();

        let monitor_labels = monitors
            .iter()
            .map(|e| 
                format!("{} ({}x{})", 
                    e.name,
                    e.resolution.0,
                    e.resolution.1
                )
            )
            .collect();

        return Ok((monitors, monitor_labels))
    }
}