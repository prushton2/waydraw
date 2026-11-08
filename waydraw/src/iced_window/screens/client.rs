use iced::Task;

use crate::iced_window::Message;

#[derive(Clone)]
pub struct Client {
    
}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: ClientMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::text("client").into()
    }
}

#[derive(Clone)]
pub enum ClientMessage {}
