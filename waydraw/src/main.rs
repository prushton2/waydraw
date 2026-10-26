pub mod config;
pub mod p2p;
pub mod encoding;
pub mod mouse;
pub mod screen_capture;

mod window;

pub use config::Config;
const TITLE: &str = constcat::concat!("Waydraw ", env!("CARGO_PKG_VERSION"));

fn main() {
    let theme = crate::config::Theme;

    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(theme)
        .subscription(window::window::subscription)
        .title(TITLE)
        .run();
}