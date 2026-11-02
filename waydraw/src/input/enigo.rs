use enigo::{Button, Coordinate::Abs, Enigo, Mouse, Settings};

pub struct EnigoInput {
    enigo: Enigo,
}

impl EnigoInput {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }
}

impl Default for EnigoInput {
    fn default() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }
}

impl super::Input for EnigoInput {
    fn click_mouse(
        &mut self,
        button: super::MouseButton,
        state:  super::MouseState,
    ) {
        let _ = self.enigo.button(
            match button {
                super::MouseButton::Left => Button::Left,
                super::MouseButton::Right => Button::Right,
            },
            match state {
                super::MouseState::Pressed => enigo::Direction::Press,
                super::MouseState::Released => enigo::Direction::Release,
            },
        );
    }

    fn move_mouse(&mut self, x: i32, y: i32) {
        self.enigo.move_mouse(x, y, Abs).unwrap();
    }
}
