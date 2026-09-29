use iced::Theme;

pub mod config;
pub mod p2p;
pub mod encoding;
pub mod mouse;

mod window;

pub use config::Config;

fn main() {
    
    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(Theme::CatppuccinFrappe)
        .subscription(window::window::subscription)
        .title("Waydraw")
        .run();
}