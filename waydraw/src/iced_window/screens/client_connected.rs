use fast_image_resize::Resizer;
use fast_image_resize::images::{Image, ImageRef};
use iced::futures::SinkExt;
use iced::widget::{button, column, image, text, text_input};
use iced::{Alignment, Length};
use iced::{
    Task,
    futures::channel::mpsc::Sender,
    widget::{MouseArea, container, row, space, stack},
};
use iced_core::image::Allocation;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use crate::iced_window::screens::client_worker::{IncomingWorkerEvent, OutgoingWorkerEvent};
use crate::iced_window::{Message, screens};
use crate::iced_window::{Screen, styles};
use crate::input::{Key, KeyState, MouseButton, MouseState};
use crate::p2p::protocol;

#[derive(Clone)]
pub enum ClientConnectedMessage {
    WorkerEvent(OutgoingWorkerEvent),
    MouseClick(MouseButton, MouseState),
    KeyEvent(iced::keyboard::Event),
    MouseMove(i32, i32),
    WindowResize(usize, usize),
    ImageAllocated(Result<Allocation, image::Error>),
    Disconnect(String),
}

pub struct ClientConnected {
    pub key: String,
    pub pin: String,
    error: String,
    sender: Option<Sender<IncomingWorkerEvent>>,
    server_info: Option<protocol::ServerHello>,
    image_allocation: Option<iced_core::image::Allocation>,

    // pub key_thread
    key_queue: Arc<Mutex<Vec<(Key, KeyState)>>>,
    key_queue_condvar: Arc<Condvar>,
    key_process_thread_active: Arc<AtomicBool>,
    key_process_thread: Option<JoinHandle<()>>,
}

impl ClientConnected {
    pub fn new(pin: String, key: String) -> Self {
        Self {
            key,
            pin,
            error: String::from(""),
            sender: None,
            server_info: None,
            image_allocation: None,

            key_queue: Arc::new(Mutex::new(vec![])),
            key_queue_condvar: Arc::new(Condvar::new()),
            key_process_thread_active: Arc::new(AtomicBool::from(false)),
            key_process_thread: None,
        }
    }

    pub fn from_key(key: String) -> Self {
        Self::new(String::from(""), key)
    }

    pub fn from_pin(pin: String) -> Self {
        Self::new(pin, String::from(""))
    }

    fn key_process_thread(
        mut sender: Sender<IncomingWorkerEvent>,
        condvar: Arc<Condvar>,
        queue: Arc<Mutex<Vec<(Key, KeyState)>>>,
        active: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        std::thread::Builder::new()
            .name("Key Process Thread".to_owned())
            .spawn(move || {
                let mut lock = queue.lock().unwrap();
                while active.load(std::sync::atomic::Ordering::Relaxed) {
                    if lock.is_empty() {
                        lock = condvar.wait(lock).unwrap();
                    }

                    while !lock.is_empty() && active.load(std::sync::atomic::Ordering::Relaxed) {
                        let (key, state) = lock.remove(0);

                        let send = sender.send(IncomingWorkerEvent::KeyEvent(key, state));
                        drop(lock);
                        let _ = iced::futures::executor::block_on(send);
                        lock = queue.lock().unwrap();
                    }
                }
            })
            .unwrap()
    }

    pub fn update(
        &mut self,
        message: ClientConnectedMessage,
        window_size: (usize, usize),
    ) -> Task<Message> {
        match message {
            ClientConnectedMessage::WorkerEvent(event) => {
                match event {
                    OutgoingWorkerEvent::UpdateKnownHosts(hostname, key) => {
                        return Task::done(Message::UpdateKnownHosts(hostname, key));
                    }
                    OutgoingWorkerEvent::Disconnect(reason) => {
                        return Task::done(Message::ClientConnectedMessage(
                            ClientConnectedMessage::Disconnect(reason),
                        ));
                    }
                    OutgoingWorkerEvent::Ready(t) => {
                        self.sender = Some(t.clone());

                        self.key_process_thread_active
                            .store(true, std::sync::atomic::Ordering::Relaxed);

                        self.key_process_thread = Some(Self::key_process_thread(
                            t,
                            self.key_queue_condvar.clone(),
                            self.key_queue.clone(),
                            self.key_process_thread_active.clone(),
                        ));
                    }
                    OutgoingWorkerEvent::UpdateKey(key) => self.key = key,
                    OutgoingWorkerEvent::UpdatePin(pin) => self.pin = pin,
                    OutgoingWorkerEvent::Failed(e) => self.error = e,
                    OutgoingWorkerEvent::SetServerInfo(e) => self.server_info = Some(e),
                    OutgoingWorkerEvent::Screenshot(frame) => {
                        let (bytes, res) = {
                            let mut lock = match frame.write() {
                                Ok(t) => t,
                                _ => return Task::none(),
                            };

                            match lock.take() {
                                Some(t) => t,
                                None => return Task::none()
                            }
                        };
                        
                        if bytes.len() != res.0 * res.1 * 4 {
                            // Decoder produced no frame (e.g. waiting for a keyframe) or size mismatch
                            return Task::none();
                        }

                        // create resizer
                        let opts = fast_image_resize::ResizeOptions::new()
                            .resize_alg(fast_image_resize::ResizeAlg::Convolution(
                                fast_image_resize::FilterType::Bilinear,
                            ))
                            .use_alpha(false);

                        let src = ImageRef::new(
                            res.0 as u32,
                            res.1 as u32,
                            &bytes,
                            fast_image_resize::PixelType::U8x4,
                        )
                        .unwrap();

                        let mut dst = Image::new(
                            window_size.0 as u32,
                            window_size.1 as u32,
                            fast_image_resize::PixelType::U8x4,
                        );
                        let _ = Resizer::new().resize(&src, &mut dst, Some(&opts));

                        let dst_bytes = dst.into_vec();

                        let handle = image::Handle::from_rgba(
                            window_size.0 as u32,
                            window_size.1 as u32,
                            dst_bytes,
                        );

                        return image::allocate(handle).map(|e| {
                            Message::ClientConnectedMessage(ClientConnectedMessage::ImageAllocated(
                                e,
                            ))
                        });
                    }
                }
                Task::none()
            }
            ClientConnectedMessage::MouseClick(button, state) => {
                if let Some(sender) = &mut self.sender {
                    let mut sender_clone = sender.clone();
                    return Task::perform(
                        async move {
                            let _ = sender_clone
                                .send(IncomingWorkerEvent::MouseClick(button, state))
                                .await;
                        },
                        |_| Message::None,
                    );
                }
                Task::none()
            }
            ClientConnectedMessage::MouseMove(x, y) => {
                if let Some(sender) = &mut self.sender {
                    let mut sender_clone = sender.clone();
                    return Task::perform(
                        async move {
                            let _ = sender_clone
                                .send(IncomingWorkerEvent::MouseMove(x, y))
                                .await;
                        },
                        |_| Message::None,
                    );
                }
                Task::none()
            }
            ClientConnectedMessage::KeyEvent(event) => {
                let (key, state): (crate::input::Key, crate::input::KeyState) = match event {
                    iced::keyboard::Event::KeyPressed {
                        key,
                        modified_key: _,
                        physical_key: _,
                        location: _,
                        modifiers: _,
                        text: _,
                        repeat: false,
                    } => (key.into(), crate::input::KeyState::Pressed),
                    iced::keyboard::Event::KeyReleased {
                        key,
                        modified_key: _,
                        physical_key: _,
                        location: _,
                        modifiers: _,
                    } => (key.into(), crate::input::KeyState::Released),
                    _ => return Task::none(),
                };

                if self
                    .key_process_thread_active
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    self.key_queue.lock().unwrap().push((key, state));
                    self.key_queue_condvar.notify_all();
                }

                Task::none()
            }
            ClientConnectedMessage::ImageAllocated(allocation) => {
                if let Ok(t) = allocation {
                    self.image_allocation = Some(t);
                }
                Task::none()
            }
            ClientConnectedMessage::WindowResize(w, h) => {
                if let Some(mut sender) = self.sender.clone() {
                    Task::perform(
                        async move {
                            let _ = sender
                                .send(IncomingWorkerEvent::SendWindowSizeToHost(w, h))
                                .await;
                        },
                        |_| Message::None,
                    )
                } else {
                    Task::none()
                }
            }
            ClientConnectedMessage::Disconnect(reason) => {
                if let Some(mut sender) = self.sender.clone() {
                    Task::perform(
                        async move {
                            let _ = sender.send(IncomingWorkerEvent::Disconnect(reason)).await;
                        },
                        |_| Message::Tab(Arc::new(move || Screen::Client(screens::Client::new()))),
                    )
                } else {
                    Task::done(Message::Tab(Arc::new(move || {
                        Screen::Client(screens::Client::new())
                    })))
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        fn textbox_row<'a>(label: &'a str, value: &'a str) -> iced::Element<'a, Message> {
            row![
                text(label)
                    .width(Length::Fixed(80.0))
                    .style(text::secondary),
                text_input("", value).style(|t, _| styles::context_textbox(t)),
            ]
            .spacing(12)
            .into()
        }

        let mut card = column![].spacing(8).align_x(Alignment::Start);

        if !self.error.is_empty() {
            card = card
                .push(text("Connect Failed").size(22))
                .push(text(&self.error).style(text::danger));

            card = card.push(
                container(button("Retry").on_press(Message::ClientConnectedMessage(
                    ClientConnectedMessage::Disconnect("Client initiated disconnect".to_string()),
                )))
                .center_x(Length::Fill)
                .padding(iced::Padding::ZERO.top(12)),
            );
        } else if self.server_info.is_none() {
            card = card.push(text("Connecting").size(22));
            if !self.key.is_empty() {
                card = card.push(textbox_row("key", &self.key));
            }
            if !self.pin.is_empty() {
                card = card.push(textbox_row("pin", &self.pin));
            }
        } else {
            return container(stack![
                MouseArea::new(row![].width(Length::Fill).height(Length::Fill))
                    .on_move(|point| Message::ClientConnectedMessage(
                        ClientConnectedMessage::MouseMove(point.x as i32, point.y as i32)
                    ))
                    .on_press(Message::ClientConnectedMessage(
                        ClientConnectedMessage::MouseClick(MouseButton::Left, MouseState::Pressed)
                    ))
                    .on_release(Message::ClientConnectedMessage(
                        ClientConnectedMessage::MouseClick(MouseButton::Left, MouseState::Released)
                    ))
                    .on_right_press(Message::ClientConnectedMessage(
                        ClientConnectedMessage::MouseClick(MouseButton::Right, MouseState::Pressed)
                    ))
                    .on_right_release(Message::ClientConnectedMessage(
                        ClientConnectedMessage::MouseClick(
                            MouseButton::Right,
                            MouseState::Released
                        )
                    )),
                match self.image_allocation.as_ref() {
                    Some(allocation) => iced::Element::from(
                        image(allocation.handle())
                            .width(Length::Fill)
                            .height(Length::Fill)
                    ),
                    None => space().width(Length::Fill).height(Length::Fill).into(),
                },
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        container(
            container(card)
                .padding(20)
                .max_width(420)
                .style(container::rounded_box),
        )
        .center(Length::Fill)
        .into()
    }
}

impl Drop for ClientConnected {
    fn drop(&mut self) {
        // Lock forces the key thread to be in the middle of a condvar wait. Then we set the thread to inactive, release the lock,
        // and tell the condvar to continue. This forces the thread to continue and die after seeing the active set to false.
        let lock = self.key_queue.lock();
        self.key_process_thread_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
        drop(lock);
        self.key_queue_condvar.notify_all();
    }
}
