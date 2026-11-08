use iced::Task;
use crate::iced_window::Message;

#[derive(Clone)]
pub struct Settings {
    
}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::text("Settings").into()
    }
}

#[derive(Clone)]
pub enum SettingsMessage {}
