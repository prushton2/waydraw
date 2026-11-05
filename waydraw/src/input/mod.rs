pub mod dummy_input;
pub mod enigo_input;

pub use dummy_input::DummyInput;
pub use enigo_input::EnigoInput;
use serde::{Deserialize, Serialize};

pub trait Input: Send {
    fn move_mouse(&mut self, x: i32, y: i32);
    fn click_mouse(&mut self, button: MouseButton, state: MouseState);
    fn key_event(&mut self, key: enigo::Key, state: KeyState);
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
}

impl From<MouseButton> for u8 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
        }
    }
}

impl TryFrom<u8> for MouseButton {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseButton::Left),
            1 => Ok(MouseButton::Right),
            _ => Err("Invalid u8".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseState {
    Released,
    Pressed,
}

impl From<MouseState> for u8 {
    fn from(value: MouseState) -> Self {
        match value {
            MouseState::Released => 0,
            MouseState::Pressed => 1,
        }
    }
}

impl TryFrom<u8> for MouseState {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseState::Released),
            1 => Ok(MouseState::Pressed),
            _ => Err("Invalid u8".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum KeyState {
    Released,
    Pressed,
}

impl From<KeyState> for u8 {
    fn from(value: KeyState) -> Self {
        match value {
            KeyState::Released => 0,
            KeyState::Pressed => 1,
        }
    }
}

impl TryFrom<u8> for KeyState {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyState::Released),
            1 => Ok(KeyState::Pressed),
            _ => Err("Invalid u8".into()),
        }
    }
}
