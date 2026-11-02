#[derive(Default)]
pub struct DummyInput;

#[allow(dead_code)]
impl DummyInput {
    pub fn new() -> Self {
        Self
    }
}

impl super::Mouse for DummyInput {
    fn click_mouse(
        &mut self,
        button: crate::p2p::protocol::mouse_click::MouseButton,
        state: crate::p2p::protocol::mouse_click::MouseState,
    ) {
        println!(
            "{} click {}",
            match button {
                crate::p2p::protocol::mouse_click::MouseButton::Left => "Left",
                crate::p2p::protocol::mouse_click::MouseButton::Right => "Right",
            },
            match state {
                crate::p2p::protocol::mouse_click::MouseState::Pressed => "Pressed",
                crate::p2p::protocol::mouse_click::MouseState::Released => "Released",
            }
        );
    }

    fn move_mouse(&mut self, x: i32, y: i32) {
        println!("Move to {}, {}", x, y);
    }
}
