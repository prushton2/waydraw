#[derive(Debug, Copy, Clone)]
pub struct MouseClick {
    pub button: MouseButton,
    pub state: MouseState
}

impl super::IntoBytes for MouseClick {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x11, 0x00];
        
        buf[1] |= u8::from(self.button);
        buf[1] <<= 1;
        buf[1] |= u8::from(self.state);
        
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let button = (bytes[0] >> 1) & 1;
        let state = bytes[0] & 1;

        Self {
            button: button.try_into().unwrap(),
            state:  state.try_into().unwrap()
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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