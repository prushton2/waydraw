pub mod window;
mod modules;

pub struct Window {
    config: crate::Config,
    ui_state: modules::ui_state::UIState
}

#[derive(Clone)]
pub enum Message {
    UIUpdate(modules::ui_state::UIUpdate),
    RemoveKnownHost(String),
    None
}