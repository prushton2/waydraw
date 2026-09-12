use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use pinray::{CaptureEvent, CaptureSession, SourceId, VideoCaptureTarget, VideoFrame};

pub struct ScreenCapture {
    _consumer_thread: JoinHandle<()>,
    latest_frame: Arc<RwLock<Option<Arc<VideoFrame>>>>,
    active: Arc<RwLock<bool>>
}

#[derive(Debug)]
pub enum ScreenCaptureError {
    BuildCaptureSessionFailed(String)
}

impl ScreenCapture {
    pub fn new(monitor_name: &str) -> Result<Self, ScreenCaptureError> {

        let latest_frame = Arc::new(RwLock::new(None));
        let active = Arc::new(RwLock::new(true));
        
        let latest_frame_clone = latest_frame.clone();
        let monitor_name_clone = monitor_name.to_owned();
        let active_clone = active.clone();

        let thread = std::thread::spawn(move || {
            let mut session = CaptureSession::builder()
                .video_target(VideoCaptureTarget::Display(SourceId::new(monitor_name_clone)))
                .pixel_format(pinray::PixelFormat::Rgb888)
                .build()
                .unwrap();

            println!("Session instantiated");
            
            let _ = session.start();

            println!("Session started");
            
            loop {
                let frame = match session.next_event(Some(std::time::Duration::from_secs(1))) {
                    Ok(CaptureEvent::Video(frame)) => frame,
                    Err(t) => {
                        println!("Error: {:?}", t);
                        continue
                    }
                    _ => continue
                };

                let mut lock = latest_frame_clone.write().unwrap();
                *lock = Some(Arc::new(frame));

                if !*active_clone.read().unwrap() {
                    return ()
                }
            }
        });
        
        
        let this = Self {
            latest_frame: latest_frame,
            _consumer_thread: thread,
            active: active
        };

        Ok(this)
    }

    /// Returns the video frame
    pub fn latest(&self) -> Option<Arc<VideoFrame>> {
        let lock = self.latest_frame.read().unwrap();

        match lock.deref() {
            Some(t) => Some(t.clone()),
            None => None
        }
    }

    pub fn kill(&self) {
        let mut lock = self.active.write().unwrap();
        *lock = false;
    }
}