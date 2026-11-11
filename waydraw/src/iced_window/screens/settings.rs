use crate::iced_window::{Message, widgets};
use iced::{
    Length, Padding, Task,
    widget::{Column, button, column, row, scrollable, space, text, text_input},
};
use native_dialog::DialogBuilder;

#[derive(Clone)]
pub struct Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(
        &mut self,
        config: &mut crate::Config,
        message: SettingsMessage,
    ) -> Task<Message> {
        match message {
            SettingsMessage::UpdateTheme(t) => {
                config.theme = t;
                config.write();
                Task::none()
            }
            SettingsMessage::UpdateKnownHostKey(old_key, new_key) => {
                if let Some(host) = config.known_hosts.remove(&old_key) {
                    config.known_hosts.insert(new_key, host);
                }
                config.write();
                Task::none()
            }
            SettingsMessage::UpdateKnownHostName(key, new_name) => {
                if let Some(host) = config.known_hosts.get_mut(&key) {
                    *host = new_name;
                }
                config.write();
                Task::none()
            }
            SettingsMessage::ResetSecretKey => {
                let confirmed = DialogBuilder::message()
                    .set_level(native_dialog::MessageLevel::Warning)
                    .set_title("Are you sure you want to reset your device ID?")
                    .set_text(
                        "Any devices that have you as a known host will need to be reconnected to.",
                    )
                    .confirm()
                    .show()
                    .unwrap_or(true); // This will fail on people who use TWMs or whatever and dont have the proper libs, and i dont really care about that

                if confirmed {
                    config.secret_key = iroh::SecretKey::generate();
                    config.overwrite_secret_key();
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(&self, config: &'a crate::Config) -> iced::Element<'a, Message> {
        let mut known_hosts = vec![];

        for (key, host) in &config.known_hosts {
            known_hosts.push(
                row![
                    text_input("Host", host).on_input(|v| Message::SettingsMessage(
                        SettingsMessage::UpdateKnownHostName(key.clone(), v)
                    )),
                    text_input("Secret Key", key).on_input(|v| Message::SettingsMessage(
                        SettingsMessage::UpdateKnownHostKey(key.clone(), v)
                    ))
                ]
                .spacing(5)
                .into(),
            );
            known_hosts.push(space().height(5).into());
        }

        if known_hosts.is_empty() {
            known_hosts =
                vec![text("No known hosts. Connect to a device and it will appear here.").into()];
        }

        scrollable(
            column![
                widgets::center_title("Settings"),
                widgets::center_separator(
                    text("Select Theme").into(),
                    widgets::theme_dropdown::<Message>(
                        &config.theme,
                        |e| Message::SettingsMessage(SettingsMessage::UpdateTheme(e))
                    )
                ),
                widgets::center_separator(
                    text("Edit known hosts").into(),
                    Column::from_vec(known_hosts).width(Length::Fill).into()
                ),
                widgets::center_separator(
                    text("Reset Device ID").into(),
                    button("Reset")
                        .on_press(Message::SettingsMessage(SettingsMessage::ResetSecretKey))
                        .into()
                ),
                widgets::center_title("Device Info"),
                widgets::center_separator(
                    text("Device").into(),
                    text(sysinfo::System::host_name().unwrap_or_default()).into()
                ),
                widgets::center_separator(
                    text("OS").into(),
                    text(sysinfo::System::name().unwrap_or_default()).into()
                ),
                widgets::center_separator(
                    text("Version").into(),
                    text(sysinfo::System::os_version().unwrap_or_default()).into()
                ),
                widgets::center_separator(
                    text("Kernel").into(),
                    text(sysinfo::System::kernel_version().unwrap_or_default()).into()
                ),
            ]
            .padding(Padding::ZERO.right(12))
            .spacing(10),
        )
        .into()
    }
}

#[derive(Clone)]
pub enum SettingsMessage {
    UpdateTheme(iced::Theme),
    /// Union types for this enum are (Old name, new name)
    UpdateKnownHostName(String, String),
    /// Union types for this enum are (Name, new key)
    UpdateKnownHostKey(String, String),
    ResetSecretKey,
}
