pub mod config;
pub mod encoding;
pub mod input;
pub mod p2p;

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
    .subscription(iced_window::window::subscription)
    .title(TITLE)
    .run();
}

pub fn get_version() -> (u8, u8, u8) {
    // split off the -rc-n tag
    let version = env!("CARGO_PKG_VERSION")
        .split(".")
        .map(|s| match s.parse::<u8>() {
            Ok(t) => t,
            Err(_) => s.split_once('-').unwrap().0.parse::<u8>().unwrap(),
        })
        .collect::<Vec<u8>>();

    (version[0], version[1], version[2])
}
