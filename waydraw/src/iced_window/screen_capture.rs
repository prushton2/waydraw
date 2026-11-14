use iced::futures::Stream;
use iced::futures::channel::mpsc::Sender;
use pinray::{CaptureEvent, CaptureSession, SourceId, VideoCaptureTarget, VideoFrame};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use crate::iced_window::Message;
use crate::iced_window::screens::host::Monitor;
use crate::iced_window::screens::host_connected::WorkerEvent;

pub struct ScreenCapture {
    _consumer_thread: JoinHandle<()>,
    latest_frame: Arc<RwLock<Option<Arc<VideoFrame>>>>,
    active: Arc<AtomicBool>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum ScreenCaptureError {
    BuildCaptureSessionFailed(String),
}

impl ScreenCapture {
    pub fn new(monitor_name: &str) -> Result<Self, ScreenCaptureError> {
        let latest_frame = Arc::new(RwLock::new(None));
        let active = Arc::new(AtomicBool::new(true));

        let this = Self {
            active: active.clone(),
            latest_frame: latest_frame.clone(),
            _consumer_thread: Self::thread(active, latest_frame, monitor_name.to_owned()),
        };

        Ok(this)
    }

    fn thread(
        active: Arc<AtomicBool>,
        latest_frame_arc: Arc<RwLock<Option<Arc<VideoFrame>>>>,
        monitor_name: String,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut session = CaptureSession::builder()
                .video_target(VideoCaptureTarget::Display(SourceId::new(monitor_name)))
                .pixel_format(pinray::PixelFormat::Rgba8888)
                .build()
                .unwrap();

            let _ = session.start();

            while active.fetch_and(true, std::sync::atomic::Ordering::Relaxed) {
                let frame = match session.next_event(Some(std::time::Duration::from_secs(1))) {
                    Ok(CaptureEvent::Video(frame)) => frame,
                    Err(t) => {
                        println!("Pinray Screencap Error: {:?}", t);
                        continue;
                    }
                    _ => continue,
                };

                let mut lock = latest_frame_arc.write().unwrap();
                *lock = Some(Arc::new(frame));
            }

            let _ = session.stop();
        })
    }

    /// Returns the video frame
    pub fn latest(&self) -> Option<Arc<VideoFrame>> {
        let lock = self.latest_frame.read().unwrap();

        lock.deref().as_ref().map(|t| t.clone())
    }
}

impl Drop for ScreenCapture {
    fn drop(&mut self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Relaxed);
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
    let mut params = params.clone();
    iced::stream::channel(100, async move |_| {
        let screencap = ScreenCapture::new(&params.monitor.id).unwrap();

        loop {
            #[cfg(not(debug_assertions))]
            tokio::time::sleep(std::time::Duration::from_millis(1000 / 30)).await;
            #[cfg(debug_assertions)]
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let frame = match screencap.latest() {
                Some(t) => t,
                None => continue,
            };

            let res = (frame.width, frame.height);

            let bytes = match frame.to_tight_bytes() {
                Some(t) => t,
                None => continue,
            };

            match params.sender.try_send(WorkerEvent::Frame(bytes, res)) {
                Ok(_) => {}
                Err(_) => continue,
            };
        }
    })
}
