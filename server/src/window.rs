use std::sync::Arc;
use std::thread::JoinHandle;

use iced::alignment::Horizontal::Center;
use tokio::sync::Mutex;

use iced::Length::Fill;
use iced::Task;
use iced::widget::{space, button, column, container, row, text, text_input};

use p2p::protocol::{FromBytes, IntoBytes, ServerInformation};
use p2p::p2p::P2PError;
use winit::monitor::MonitorHandle;

use crate::mouse::{self, Mouse};

pub struct Window {
    p2p: Arc<Mutex<Option<p2p::P2P>>>,
    // mouse_move_handle: Option<JoinHandle<i32>>,
    
    available_monitors: Vec<MonitorHandle>,
    selected_monitor: Option<usize>,

    pin: Option<String>,
    key: Option<String>,
    
    wait_reason: String,
    error: String,


    // AAAA
    labels: Vec<String>
}

#[derive(Clone)]
pub enum Message {
    Register,
    AwaitClient(Result<(Arc<Mutex<Option<p2p::P2P>>>, String, String), Arc<P2PError>>),
    ConnectionEstablished(Result<(), String>),
    SelectMonitor(usize),
    Disconnect,
    Drop,
    Null(())
}

impl Window {
    pub fn boot(monitors: Vec<MonitorHandle>) -> Self {
        let monitor_labels = monitors
            .iter()
            .map(|e| 
                format!("{} ({}x{}@{}hz)", 
                    e.name().unwrap_or("".to_owned()), 
                    e.size().width, 
                    e.size().height, 
                    e.refresh_rate_millihertz().unwrap_or(0) / 1000
                )
            ).collect();

        let this = Self {
            p2p: Arc::new(Mutex::new(None)),
            // mouse_move_handle: None,

            available_monitors: monitors,
            selected_monitor: None,

            pin: None,
            key: None,

            wait_reason: String::from(""),
            error: String::from(""),

            labels: monitor_labels,
        };

        this
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::Register => {
                if self.selected_monitor.is_none() {
                    self.error = "Error: Please select a monitor".to_owned();
                    return Task::none()
                }
                
                self.error = "".to_owned();
                self.wait_reason = "Registering...".to_owned();

                Task::perform(
                    async move {
                        let (server, key) = p2p::P2P::init().await.map_err(|e| Arc::new(e))?;

                        let pin = p2p::remote_key_store::generate_key();
                        let key = key.to_string();
                        p2p::remote_key_store::set(&pin, &key.to_string()).await;

                        Ok((Arc::new(Mutex::new(Some(server))), key, pin))
                    },
                    Message::AwaitClient
                )
            },
            Message::AwaitClient(result) => {
                let (p2p, key, pin) = match result {
                    Ok(t) => t,
                    Err(t) => {
                        self.error = String::from((*t).clone());
                        self.wait_reason = String::from("");
                        return Task::none()
                    }
                };

                self.p2p = p2p;
                self.key = Some(key);
                self.pin = Some(pin);

                let arc = self.p2p.clone();
                
                self.wait_reason = "Waiting for connection".to_owned();
                Task::perform(
                async move {
                        let mut lock = arc.lock().await;
                        let p2p = lock.as_mut().unwrap();
                        let _ = p2p.await_connection().await;
                        Ok(())
                    },
                    Message::ConnectionEstablished,
                )
            }
            Message::ConnectionEstablished(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                self.wait_reason = "Connection Established".to_owned();
                let selected_monitor = &self.available_monitors[self.selected_monitor.unwrap()];

                // construct server info to send to client
                let p2p_arc = self.p2p.clone();
                let server_info_bytes = ServerInformation {
                    width:  selected_monitor.size().width,
                    height: selected_monitor.size().height
                }.into_bytes();

                // the coordinates of the top left of the monitor to offset mouse_move
                let coordinates = selected_monitor.position().clone();

                // let handle: JoinHandle<i32> = std::thread::spawn(move || {
                //     tokio::runtime::Runtime::new().unwrap().block_on(async move { 
                        
                //         let mut server_lock = p2p_arc.lock().await;
                //         let server = server_lock.as_mut().unwrap();

                //         let _ = server.send(&server_info_bytes).await;

                //         drop(server_lock);

                //         let mut mouse: Box<dyn Mouse> = Box::new(mouse::DummyMouse::new());
                //         if !cfg!(debug_assertions) {
                //             mouse = Box::new(mouse::EnigoMouse::new());
                //         }
                        
                //         loop {
                //             let mut server_lock = p2p_arc.lock().await;
                //             let server = server_lock.as_mut().unwrap();

                //             let response = server.read().await.unwrap();
                            
                //             // acquire and drop lock rapidly to let other threads use lock if needed
                //             drop(server_lock); 
                            
                //             let parsed = p2p::protocol::FromBytes::parse(&response);
                //             match parsed {
                //                 FromBytes::MouseMove(message) => {
                //                     mouse.move_mouse(coordinates.x as u32 + message.x, coordinates.y as u32 + message.y);
                //                 }
                //                 FromBytes::MouseClick(message) => {
                //                     mouse.click_mouse(message.button, message.state);
                //                 }
                //                 _ => {}
                //             }
                //         }
                //     })
                // });

                // self.mouse_move_handle = Some(handle);

                Task::none()
            },
            Message::Disconnect => {
                // self.mouse_move_handle = None;
                self.pin = None;
                self.key = None;
                self.error = String::from("");
                self.wait_reason = String::from("");
                let p2p_arc = self.p2p.clone();
                self.p2p = Arc::new(Mutex::new(None));

                Task::perform(
                async move {
                        let mut lock = p2p_arc.lock().await;
                        if let Some(p2p) = lock.as_mut() {
                            let _ = p2p.close().await;
                        } else {
                            println!("Could not close connection");
                        }
                        
                        ()
                    },
                    Message::Null,
                )
            },
            Message::SelectMonitor(i) => {
                self.selected_monitor = Some(i);
                Task::none()
            }
            Message::Drop => {
                Task::none()
            },
            Message::Null(_) => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        // if self.mouse_move_handle.is_some() {
        //     return button("Disconnect").on_press(Message::Disconnect).into();
        // }

        let pin = self.pin.clone().unwrap_or("".to_owned());
        let key = self.key.clone().unwrap_or("".to_owned());

        let mut buttons: Vec<iced::Element<'_, Message>> = vec![];

        for i in 0..self.labels.len() {
            let mut button = button(self.labels[i].as_str()).on_press(Message::SelectMonitor(i)).width(Fill);

            if Some(i) == self.selected_monitor {
                button = button.style(|theme, status| {
                    button::subtle(theme, status)
                });
            }

            buttons.push(
                button.into()
            );
            buttons.push(space().height(5).into())
        }

        container (
            column![
                text("Select a monitor").width(Fill).align_x(Center),
                iced::widget::Column::from_vec(buttons).width(Fill).align_x(Center),
                
                space().height(20),
                container(button("Allow Connections").on_press(Message::Register)).center_x(Fill),
                space().height(20),
                
                row![text("Pin"), space().width(24), text_input(&pin, &pin).on_input(|_| Message::Drop)],
                row![text("Key"), space().width(20), text_input(&key, &key).on_input(|_| Message::Drop)],
                
                text(&self.wait_reason).width(Fill).align_x(Center),
                text(&self.error).width(Fill).align_x(Center).style(|t| {text::danger(t)}),
            ]
            .max_width(400)

        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}


