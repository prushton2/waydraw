use iced::futures::channel::mpsc::Sender;
use iced::futures::{SinkExt, Stream};
use pinray::{CaptureEvent, CaptureSession, SourceId, VideoCaptureTarget, VideoFrame};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use crate::iced_window::Message;
use crate::iced_window::screens::host::Monitor;
use crate::iced_window::screens::host_connected::WorkerEvent;

pub struct ScreenCapture {
    _consumer_thread: JoinHandle<()>,
    latest_frame: Arc<RwLock<Option<Arc<VideoFrame>>>>,
    active: Arc<RwLock<bool>>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum ScreenCaptureError {
    BuildCaptureSessionFailed(String),
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
                .video_target(VideoCaptureTarget::Display(SourceId::new(
                    monitor_name_clone,
                )))
                .pixel_format(pinray::PixelFormat::Rgba8888)
                .build()
                .unwrap();

            let _ = session.start();

            loop {
                let frame = match session.next_event(Some(std::time::Duration::from_secs(1))) {
                    Ok(CaptureEvent::Video(frame)) => frame,
                    Err(t) => {
                        println!("Error: {:?}", t);
                        continue;
                    }
                    _ => continue,
                };

                let mut lock = latest_frame_clone.write().unwrap();
                *lock = Some(Arc::new(frame));

                if !*active_clone.read().unwrap() {
                    return;
                }
            }
        });

        let this = Self {
            latest_frame,
            _consumer_thread: thread,
            active,
        };

        Ok(this)
    }

    /// Returns the video frame
    pub fn latest(&self) -> Option<Arc<VideoFrame>> {
        let lock = self.latest_frame.read().unwrap();

        lock.deref().as_ref().map(|t| t.clone())
    }

    pub fn kill(&self) {
        let mut lock = self.active.write().unwrap();
        *lock = false;
    }
}

impl Drop for ScreenCapture {
    fn drop(&mut self) {
        self.kill();
    }
}

#[derive(Clone)]
pub struct Params {
    pub monitor: Monitor,
    pub sender: Sender<WorkerEvent>,
}

impl Hash for Params {
    fn hash<H: Hasher>(&self, h: &mut H) {
        "screencap_stream".hash(h)
    }
}

pub fn worker(params: &Params) -> impl Stream<Item = Message> + use<> {
    println!("Worker initialized");
    let mut params = params.clone();
    iced::stream::channel(100, async move |_| {
        let screencap = ScreenCapture::new(&params.monitor.label).unwrap();

        loop {
            #[cfg(not(debug_assertions))]
            tokio::time::sleep(std::time::Duration::from_millis(1000 / 30)).await;
            #[cfg(debug_assertions)]
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            println!("Reading frame");

            let frame = match screencap.latest() {
                Some(t) => t,
                None => continue,
            };
            println!("  Frame exists");

            let bytes = match frame.to_tight_bytes() {
                Some(t) => t,
                None => continue,
            };
            println!("  Tight bytes exists");

            match params.sender.try_send(WorkerEvent::Frame(bytes)) {
                Ok(_) => {}
                Err(e) => println!("Send error: {}", e.to_string()),
            };
            println!("  Sent");
        }
    })
}
