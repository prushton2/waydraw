use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, waydraw_macros::IntoBytes, Serialize, Deserialize)]
#[opcode(0x11)]
pub struct MouseClick {
    pub button: MouseButton,
    pub state: MouseState
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right
}

impl From<MouseButton> for u8 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Right => 1
        }
    }
}

impl TryFrom<u8> for MouseButton {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseButton::Left),
            1 => Ok(MouseButton::Right),
            _ => Err("Invalid u8".into())
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseState {
    Released,
    Pressed
}

impl From<MouseState> for u8 {
    fn from(value: MouseState) -> Self {
        match value {
            MouseState::Released => 0,
            MouseState::Pressed => 1
        }
    }
}

impl TryFrom<u8> for MouseState {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseState::Released),
            1 => Ok(MouseState::Pressed),
            _ => Err("Invalid u8".into())
        }
    }
}