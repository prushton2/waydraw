// use iced::Length::Fill;
// use iced::Task;
use iced::widget::button;
// use iced::alignment::Horizontal::Center;

use super::Window;
use super::Message;

impl Window {
    pub fn boot() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        return button("Disconnect").into();
    }
}