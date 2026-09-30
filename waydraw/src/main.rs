pub mod config;
pub mod p2p;
pub mod encoding;
pub mod mouse;
pub mod screen_capture;

mod window;

pub use config::Config;

fn main() {
    let theme = crate::config::Theme;

    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(theme)
        .subscription(window::window::subscription)
        .title("Waydraw")
        .run();
}