use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use pinray::{CaptureEvent, CaptureSession, SourceId, VideoCaptureTarget, VideoFrame};

pub struct ScreenCapture {
    consumer_thread: JoinHandle<()>,
    latest_frame: Arc<RwLock<Option<Arc<VideoFrame>>>>,
}

#[derive(Debug)]
pub enum ScreenCaptureError {
    BuildCaptureSessionFailed(String)
}

impl ScreenCapture {
    pub fn new(monitor_name: &str) -> Result<Self, ScreenCaptureError> {

        let latest_frame = Arc::new(RwLock::new(None));
        
        let latest_frame_clone = latest_frame.clone();
        let monitor_name_clone = monitor_name.to_owned();

        let thread = std::thread::spawn(move || {
            let mut session = CaptureSession::builder()
                .video_target(VideoCaptureTarget::Display(SourceId::new(monitor_name_clone)))
                .build()
                .unwrap();
        
            let _ = session.start();
            
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
            }
            
        });
        
        
        let this = Self {
            latest_frame: latest_frame,
            consumer_thread: thread
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
}