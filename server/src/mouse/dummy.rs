use p2p::protocol::mouse_click::{MouseButton, MouseState};
#[allow(dead_code)]
pub struct DummyMouse;

#[allow(dead_code)]
impl DummyMouse {
    pub fn new() -> Self {
        Self
    }
}

impl super::Mouse for DummyMouse {
    fn click_mouse(&mut self, _button: p2p::protocol::mouse_click::MouseButton, state: p2p::protocol::mouse_click::MouseState) {
        // println!("{} click {}",
        //     match button {
        //         MouseButton::Left => "Left",
        //         MouseButton::Right => "Right"
        //     },
        //     match state {
        //         MouseState::Pressed => "Pressed",
        //         MouseState::Released => "Released"
        //     }
        // );
    }
    
    fn move_mouse(&mut self, _x: i32, _y: i32) {
        // println!("Move to {}, {}", x, y);
    }
}