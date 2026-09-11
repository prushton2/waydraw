use std::time::{Duration, Instant};
use xcap::Monitor;

fn main() {
    env_logger::try_init().ok();
    let monitors = Monitor::all().unwrap();
    for m in &monitors {
        println!("monitor: {:?} {}x{}", m.name(), m.width().unwrap_or(0), m.height().unwrap_or(0));
    }
    let (recorder, rx) = monitors[0].video_recorder().unwrap();
    println!("recorder created");
    recorder.start().unwrap();
    println!("started");

    let start = Instant::now();
    let mut n = 0;
    loop {
        match rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(f) => { n += 1; println!("frame {} {}x{} bytes={}", n, f.width, f.height, f.raw.len()); }
            Err(e) => println!("no frame after {:?}: {:?}", start.elapsed(), e),
        }
        if start.elapsed() > Duration::from_secs(10) { break; }
    }
    println!("total frames: {}", n);
}
