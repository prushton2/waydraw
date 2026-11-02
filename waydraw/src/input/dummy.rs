#[derive(Default)]
pub struct DummyInput;

#[allow(dead_code)]
impl DummyInput {
    pub fn new() -> Self {
        Self
    }
}

impl super::Input for DummyInput {
    fn click_mouse(
        &mut self,
        button: crate::input::MouseButton,
        state:  crate::input::MouseState,
    ) {
        println!(
            "{} click {}",
            match button {
                crate::input::MouseButton::Left => "Left",
                crate::input::MouseButton::Right => "Right",
            },
            match state {
                crate::input::MouseState::Pressed => "Pressed",
                crate::input::MouseState::Released => "Released",
            }
        );
    }

    fn move_mouse(&mut self, x: i32, y: i32) {
        println!("Move to {}, {}", x, y);
    }
}
