use iced::Theme;

mod window;

// #[tokio::main]
fn main() {
    let _ = iced::application(window::Window::boot, window::Window::update, window::Window::view)
        .subscription(window::subscription)
        // .subscription(window::p2p_loop_subscription)
        .theme(Theme::CatppuccinFrappe)
        .title("Waydraw Client")
        .run();
}