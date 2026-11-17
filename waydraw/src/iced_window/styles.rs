use iced::{
    Border, Theme,
    border::Radius,
    widget::{button, text_input},
};

pub fn invisible_text_input(theme: &Theme) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(theme.palette().background),
        border: Border {
            color: theme.palette().background,
            width: 0.0,
            radius: Radius::new(0),
        },
        icon: theme.palette().background,
        placeholder: theme.palette().background,
        value: theme.palette().background,
        selection: theme.palette().background,
    }
}
pub fn client_host_selected(theme: &Theme) -> button::Style {
    button::Style {
        background: None,
        text_color: theme.extended_palette().primary.base.color,
        border: Border {
            color: theme.palette().background,
            width: 0.0,
            radius: Radius::new(0),
        },
        shadow: iced::Shadow {
            color: theme.palette().primary,
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

pub fn client_host_unselected(theme: &Theme) -> button::Style {
    button::Style {
        background: Some(iced::Background::from(theme.palette().primary)),
        text_color: theme.extended_palette().primary.base.text,
        border: Border {
            color: iced::Color::from_rgb(0.0, 0.0, 0.0),
            width: 0.0,
            radius: Radius::new(0),
        },
        shadow: iced::Shadow {
            color: theme.palette().primary,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

pub fn square_button(theme: &Theme) -> button::Style {
    button::Style {
        background: Some(iced::Background::from(theme.palette().primary)),
        text_color: theme.extended_palette().primary.base.text,
        border: Border {
            color: iced::Color::from_rgb(0.0, 0.0, 0.0),
            width: 0.0,
            radius: Radius::new(0),
        },
        shadow: iced::Shadow {
            color: theme.palette().background,
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

pub fn selected_monitor(theme: &Theme) -> button::Style {
    button::Style {
        background: Some(iced::Background::from(theme.palette().success)),
        text_color: theme.extended_palette().primary.base.text,
        border: Border {
            color: iced::Color::from_rgb(0.0, 0.0, 0.0),
            width: 0.0,
            radius: Radius::new(2),
        },
        shadow: iced::Shadow {
            color: theme.palette().background,
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

pub fn unselected_monitor(theme: &Theme) -> button::Style {
    button::Style {
        background: Some(iced::Background::from(theme.palette().primary)),
        text_color: theme.extended_palette().primary.base.text,
        border: Border {
            color: iced::Color::from_rgb(0.0, 0.0, 0.0),
            width: 0.0,
            radius: Radius::new(2),
        },
        shadow: iced::Shadow {
            color: theme.palette().background,
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        snap: false,
    }
}

pub fn context_textbox(theme: &Theme) -> text_input::Style {
    text_input::Style {
        background: iced::Background::from(theme.extended_palette().background.weak.color),
        border: Border {
            color: theme.extended_palette().primary.base.text,
            width: 0.0,
            radius: Radius::new(2),
        },
        icon: iced::Color::from_rgb8(255, 255, 255),
        placeholder: iced::Color::from_rgb8(0, 0, 0),
        selection: theme.extended_palette().primary.weak.color,
        value: theme.extended_palette().background.weak.text,
    }
}
