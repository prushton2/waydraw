use iced::Theme;

use winit::monitor::MonitorHandle;
use xcap;

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
        .title("Waydraw Server")
        .run();
}

// use fs_extra::dir;
// use std::time::Instant;
// use xcap::Monitor;

// fn normalized(filename: String) -> String {
//     filename.replace(['|', '\\', ':', '/'], "")
// }

// fn main() {
//     let start = Instant::now();
//     let monitors = Monitor::all().unwrap();

//     dir::create_all("target/monitors", true).unwrap();

//     for monitor in monitors {
//         let image = monitor.capture_image().unwrap();

//         image
//             .save(format!(
//                 "target/monitors/monitor-{}.png",
//                 normalized(monitor.friendly_name().unwrap())
//             ))
//             .unwrap();
//     }

//     println!("Elapsed time: {:?}", start.elapsed());
// }