pub mod config;
pub mod encoding;
pub mod input;
pub mod p2p;
pub mod screen_capture;

mod iced_window;

pub use config::Config;
const TITLE: &str = constcat::concat!("Waydraw ", env!("CARGO_PKG_VERSION"));

fn main() {
    let theme = crate::config::Theme;

    let _ = iced::application(
        iced_window::Window::boot,
        iced_window::Window::update,
        iced_window::Window::view,
    )
    .theme(theme)
    // .subscription(iced_window::window::subscription)
    .title(TITLE)
    .run();
}
