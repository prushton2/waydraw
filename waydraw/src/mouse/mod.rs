use crate::p2p::protocol::mouse_click::{MouseButton, MouseState};

pub mod dummy;
pub mod enigo;

#[allow(unused)]
pub use dummy::DummyMouse;
#[allow(unused)]
pub use enigo::EnigoMouse;

pub trait Mouse: Send {
    fn move_mouse(&mut self, x: i32, y: i32);
    fn click_mouse(&mut self, button: MouseButton, state: MouseState);
}
