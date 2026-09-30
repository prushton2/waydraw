pub mod config;
pub mod p2p;
pub mod encoding;
pub mod mouse;
pub mod screen_capture;

mod window;

pub use config::Config;

fn main() {
    let config = Config::load_or_generate();
    
    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(config.theme)
        .subscription(window::window::subscription)
        .title("Waydraw")
        .run();
}