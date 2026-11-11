pub mod dummy_input;
pub mod enigo_input;

pub use dummy_input::DummyInput;
pub use enigo_input::EnigoInput;
use serde::{Deserialize, Serialize};

pub trait Input: Send + Sync {
    fn move_mouse(&mut self, x: i32, y: i32);
    fn click_mouse(&mut self, button: MouseButton, state: MouseState);
    fn key_event(&mut self, key: Key, state: KeyState);
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseState {
    Released,
    Pressed,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum KeyState {
    Released,
    Pressed,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Key {
    Unicode(char),
    Named(Named),
    Other(u32),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Named {
    Super,
    Alt,
    Ctrl,
    Shift,
    Space,
    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,
    Backspace,
    Enter,
    Delete,
    Tab,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl From<iced::keyboard::Key> for Key {
    fn from(value: iced::keyboard::Key) -> Self {
        match value {
            iced::keyboard::Key::Character(c) => Self::Unicode(c.chars().next().unwrap_or('\0')),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                Self::Named(Named::ArrowUp)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                Self::Named(Named::ArrowDown)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                Self::Named(Named::ArrowLeft)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                Self::Named(Named::ArrowRight)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Super) => {
                Self::Named(Named::Super)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Control) => {
                Self::Named(Named::Ctrl)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Alt) => Self::Named(Named::Alt),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Shift) => {
                Self::Named(Named::Shift)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) => {
                Self::Named(Named::Space)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
                Self::Named(Named::Backspace)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) => {
                Self::Named(Named::Delete)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                Self::Named(Named::Enter)
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => Self::Named(Named::Tab),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F1) => Self::Named(Named::F1),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F2) => Self::Named(Named::F2),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F3) => Self::Named(Named::F3),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F4) => Self::Named(Named::F4),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F5) => Self::Named(Named::F5),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F6) => Self::Named(Named::F6),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F7) => Self::Named(Named::F7),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F8) => Self::Named(Named::F8),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F9) => Self::Named(Named::F9),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F10) => Self::Named(Named::F10),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F11) => Self::Named(Named::F11),
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F12) => Self::Named(Named::F12),

            _ => Key::Other(0),
        }
    }
}

impl From<Key> for enigo::Key {
    fn from(value: Key) -> Self {
        match value {
            Key::Unicode(c) => enigo::Key::Unicode(c),
            Key::Other(c) => enigo::Key::Other(c),

            Key::Named(Named::ArrowUp) => enigo::Key::DownArrow,
            Key::Named(Named::ArrowDown) => enigo::Key::UpArrow,
            Key::Named(Named::ArrowLeft) => enigo::Key::LeftArrow,
            Key::Named(Named::ArrowRight) => enigo::Key::RightArrow,
            Key::Named(Named::Super) => enigo::Key::Meta,
            Key::Named(Named::Ctrl) => enigo::Key::Control,
            Key::Named(Named::Alt) => enigo::Key::Alt,
            Key::Named(Named::Shift) => enigo::Key::Shift,
            Key::Named(Named::Space) => enigo::Key::Space,
            Key::Named(Named::Backspace) => enigo::Key::Backspace,
            Key::Named(Named::Delete) => enigo::Key::Delete,
            Key::Named(Named::Enter) => enigo::Key::Return,
            Key::Named(Named::Tab) => enigo::Key::Tab,
            Key::Named(Named::F1) => enigo::Key::F1,
            Key::Named(Named::F2) => enigo::Key::F2,
            Key::Named(Named::F3) => enigo::Key::F3,
            Key::Named(Named::F4) => enigo::Key::F4,
            Key::Named(Named::F5) => enigo::Key::F5,
            Key::Named(Named::F6) => enigo::Key::F6,
            Key::Named(Named::F7) => enigo::Key::F7,
            Key::Named(Named::F8) => enigo::Key::F8,
            Key::Named(Named::F9) => enigo::Key::F9,
            Key::Named(Named::F10) => enigo::Key::F10,
            Key::Named(Named::F11) => enigo::Key::F11,
            Key::Named(Named::F12) => enigo::Key::F12,
        }
    }
}
