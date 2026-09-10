use iced::Theme;

use winit::monitor::MonitorHandle;
// use xcap;

mod mouse;
mod window;

fn main() {
    
    let _ = iced::application(
        || {
            let displays = winit::event_loop::EventLoop::new().unwrap().available_monitors().collect::<Vec<MonitorHandle>>();
            window::Window::boot(displays)
        },
        window::Window::update,
        window::Window::view
    )
        .theme(Theme::CatppuccinFrappe)
        .subscription(window::subscriptions::subscription)
        .title("Waydraw Server")
        .run();
}

