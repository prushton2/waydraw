use crate::iced_window::{Message, styles};
use iced::{
    Alignment, Length, Task,
    widget::{button, column, container, row, space, text, text_input},
};

#[derive(Clone)]
pub struct Host {
    monitors: Vec<(Monitor, String)>,
    monitor: Monitor,
    pin: String,
    key: String,
    wait: String,
    err: String,
}

impl Host {
    pub fn new() -> Self {
        let monitors: Vec<(Monitor, String)> = read_monitors()
            .unwrap()
            .into_iter()
            .map(|e| {
                (
                    e.clone(),
                    format!("{} {}x{}", e.id, e.resolution.0, e.resolution.1),
                )
            })
            .collect();

        Self {
            monitor: monitors[0].0.clone(),
            monitors,
            pin: String::from(""),
            key: String::from(""),
            wait: String::from(""),
            err: String::from(""),
        }
    }

    pub fn update(&mut self, message: HostMessage) -> Task<Message> {
        match message {
            HostMessage::SetMonitor(m) => {
                self.monitor = m;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let buttons = self
            .monitors
            .iter()
            .map(|e| {
                button(e.1.as_str())
                    .on_press(Message::HostMessage(HostMessage::SetMonitor(e.0.clone())))
                    .style(|t, _| {
                        if self.monitor == e.0 {
                            styles::selected_monitor(t)
                        } else {
                            styles::unselected_monitor(t)
                        }
                    })
                    .into()
            })
            .collect();

        column![
            text("Select a monitor")
                .width(Length::Fill)
                .align_x(Alignment::Center),
            iced::widget::Column::from_vec(buttons)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            space().height(20),
            row![
                space().width(Length::Fill),
                button("Allow Connections").on_press(Message::Tab(
                    crate::iced_window::Screen::HostConnected(super::HostConnected::new(
                        self.monitor.clone()
                    ))
                )),
                space().width(Length::Fill),
            ]
            .width(Length::Fill),
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
pub enum HostMessage {
    SetMonitor(Monitor),
}

#[derive(Clone, PartialEq)]
pub struct Monitor {
    pub id: String,
    pub position: (i32, i32),
    pub resolution: (u32, u32),
    pub scale: f32,
    pub label: String,
}

fn read_monitors() -> Result<Vec<Monitor>, String> {
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
                .find(|e| e.name == name || e.id.to_string() == name)
                .unwrap();

            Monitor {
                id: pinray_source.id.0.clone(),
                position: (displayinfo_source.x, displayinfo_source.y),
                resolution: (pinray_source.width, pinray_source.height),
                label: format!("{} {}x{}", pinray_source.name, pinray_source.width, pinray_source.height),
                #[cfg(target_os = "macos")] // Macos reports points, not pixels, which are already scaled, so we set the scale to 1.0
                scale: 1.0,
                #[cfg(not(target_os = "macos"))]
                scale: pinray_source.scale_factor_milli as f32 / 1000.0,
            }
        })
        .collect::<Vec<Monitor>>();

    Ok(monitors)
}
