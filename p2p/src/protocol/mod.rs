pub mod server_hello;
pub mod client_hello;
pub mod mouse_move;
pub mod mouse_click;
pub mod window_resized;

pub use server_hello::ServerHello;
pub use client_hello::ClientHello;
pub use mouse_move::MouseMove;
pub use mouse_click::MouseClick;
pub use window_resized::WindowResized;

#[derive(Debug, Clone)]
pub enum FromBytes {
    ServerHello(ServerHello),
    ClientHello(ClientHello),
    MouseMove(MouseMove),
    MouseClick(MouseClick),
    WindowResized(WindowResized),
    UnknownInstruction(Vec<u8>)
}

/*
    Opcode Categories

    0x0[0-F]: Client / Server metadata
    0x1[0-F]: Mouse events

*/

impl FromBytes {
    pub fn parse(bytes: &[u8]) -> Self {
        match bytes[0] {
            0x00 => Self::ServerHello(ServerHello::from_bytes(&bytes[1..])),
            0x01 => Self::ClientHello(ClientHello::from_bytes(&bytes[1..])),
            0x02 => Self::WindowResized(WindowResized::from_bytes(&bytes[1..])),
            0x10 => Self::MouseMove(MouseMove::from_bytes(&bytes[1..])),
            0x11 => Self::MouseClick(MouseClick::from_bytes(&bytes[1..])),
            _ => Self::UnknownInstruction(bytes.into())
        }
    }
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}