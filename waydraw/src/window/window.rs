use crate::window::modules::*;

use super::Window;
use super::Message;

impl Window {
    pub fn boot() -> Self {
        Self {
            config: crate::Config::load_or_generate(),
            ui_state: ui_state::UIState::Host { pin: String::from(""), key: String::from(""), wait: String::from(""), error: String::from("") }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UIUpdate(update) => {
                ui_state::update(self, update);
            },
            Message::RemoveKnownHost(host) => {
                self.config.known_hosts.remove(&host);
                self.config.write();
            }
            Message::None => {},
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        return ui_state::view(self);
    }
}