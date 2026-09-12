use enigo::Mouse;
use enigo::{Button, Coordinate::Abs, Enigo, Settings};
use p2p::protocol::mouse_click::{MouseButton, MouseState};
pub struct EnigoMouse {
    enigo: Enigo
}

impl EnigoMouse {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap()
        }
    }
}

impl super::Mouse for EnigoMouse {
    fn click_mouse(&mut self, button: p2p::protocol::mouse_click::MouseButton, state: p2p::protocol::mouse_click::MouseState) {
        let _ = self.enigo.button(
            match button {
                MouseButton::Left => Button::Left,
                MouseButton::Right => Button::Right,
            },
            match state {
                MouseState::Pressed => enigo::Direction::Press,
                MouseState::Released => enigo::Direction::Release
            });
    }
    
    fn move_mouse(&mut self, x: i32, y: i32) {
        self.enigo.move_mouse(x, y, Abs).unwrap();
    }
}