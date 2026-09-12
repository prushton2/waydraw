use iced::Theme;

mod mouse;
mod window;
mod screen_grabber;

fn main() {
    
    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(Theme::CatppuccinFrappe)
        .subscription(window::subscriptions::subscription)
        .title("Waydraw Server")
        .run();
}

