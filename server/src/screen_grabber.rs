use std::sync::{Arc, Mutex};
use std::thread;
use xcap::{Frame, Monitor, VideoRecorder, XCapResult};

pub struct ScreenGrabber {
    recorder: VideoRecorder,
    latest: Arc<Mutex<Option<Arc<Frame>>>>,
}

impl ScreenGrabber {
    pub fn new(monitor_name: &str) -> XCapResult<Self> {
        
        let monitors = Monitor::all().unwrap();
        let mut selected_monitor: &Monitor = &monitors[0];
        
        for current_monitor in &monitors {
            if current_monitor.name().unwrap() == monitor_name {
                selected_monitor = current_monitor;
                break;
            }
        }
        let (recorder, rx) = selected_monitor.video_recorder()?;
        let latest = Arc::new(Mutex::new(None));

        let slot = latest.clone();
        thread::spawn(move || {
            // Read constantly so the capture thread never blocks
            // and frames never pile up.
            for frame in rx {
                *slot.lock().unwrap() = Some(Arc::new(frame));
            }
        });

        recorder.start()?;
        Ok(Self { recorder, latest })
    }

    /// Near-instant: just clones an Arc.
    pub fn latest(&self) -> Option<Arc<Frame>> {
        self.latest.lock().unwrap().clone()
    }

    pub fn pause(&self) -> XCapResult<()> { self.recorder.stop() }
    pub fn resume(&self) -> XCapResult<()> { self.recorder.start() }
}