use p2p::protocol::mouse_click::{MouseButton, MouseState};

pub mod enigo;
pub mod dummy;

#[allow(unused)]
pub use enigo::EnigoMouse;
#[allow(unused)]
pub use dummy::DummyMouse;

pub trait Mouse: Send {
    fn move_mouse(&mut self, x: u32, y: u32);
    fn click_mouse(&mut self, button: MouseButton, state: MouseState);
}