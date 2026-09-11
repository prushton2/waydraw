pub mod server_hello;
pub mod client_hello;
pub mod mouse_move;
pub mod mouse_click;
pub mod window_resized;
pub mod screenshot;
pub mod compressed_screenshot;

pub use server_hello::ServerHello;
pub use client_hello::ClientHello;
pub use mouse_move::MouseMove;
pub use mouse_click::MouseClick;
pub use window_resized::WindowResized;
pub use screenshot::Screenshot;
pub use compressed_screenshot::CompressedScreenshot;

#[derive(Debug, Clone)]
pub enum FromBytes {
    ServerHello(ServerHello),
    ClientHello(ClientHello),
    MouseMove(MouseMove),
    MouseClick(MouseClick),
    WindowResized(WindowResized),
    Screenshot(Screenshot),
    CompressedScreenshot(CompressedScreenshot),
    UnknownInstruction(Vec<u8>),
}

/*
    Opcode Categories

    0x0[0-F]: Client / Server metadata
    0x1[0-F]: Mouse events
    0x2[0-F]: Screenshot events

*/

impl FromBytes {
    pub fn parse(bytes: &[u8]) -> Self {
        match bytes[0] {
            0x00 => Self::ServerHello(ServerHello::from_bytes(&bytes[1..])),
            0x01 => Self::ClientHello(ClientHello::from_bytes(&bytes[1..])),
            0x02 => Self::WindowResized(WindowResized::from_bytes(&bytes[1..])),
            0x10 => Self::MouseMove(MouseMove::from_bytes(&bytes[1..])),
            0x11 => Self::MouseClick(MouseClick::from_bytes(&bytes[1..])),
            0x20 => Self::Screenshot(Screenshot::from_bytes(&bytes[1..])),
            0x21 => Self::CompressedScreenshot(CompressedScreenshot::from_bytes(&bytes[1..])),
            _ => Self::UnknownInstruction(bytes.into())
        }
    }
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}