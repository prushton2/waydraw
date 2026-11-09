use iced::{Alignment, Length, widget::{column, pick_list, row, space, text, text_input}};

pub fn theme_dropdown<T: Clone + 'static>(theme: &iced::Theme, message: fn(iced::Theme) -> T) -> iced::Element<'static, T> {
    let options = [
        iced::Theme::Light,
        iced::Theme::Dark,
        iced::Theme::Dracula,
        iced::Theme::Nord,
        iced::Theme::SolarizedLight,
        iced::Theme::SolarizedDark,
        iced::Theme::GruvboxLight,
        iced::Theme::GruvboxDark,
        iced::Theme::CatppuccinLatte,
        iced::Theme::CatppuccinFrappe,
        iced::Theme::CatppuccinMacchiato,
        iced::Theme::CatppuccinMocha,
        iced::Theme::TokyoNight,
        iced::Theme::TokyoNightStorm,
        iced::Theme::TokyoNightLight,
        iced::Theme::KanagawaWave,
        iced::Theme::KanagawaDragon,
        iced::Theme::KanagawaLotus,
        iced::Theme::Moonfly,
        iced::Theme::Nightfly,
        iced::Theme::Oxocarbon,
        iced::Theme::Ferra,
    ];

    pick_list(options, Some(theme.clone()), message).into()
}


pub fn center_separator<'a, T: Clone + 'a>(
    left: iced::Element<'a, T>,
    right: iced::Element<'a, T>,
) -> iced::Element<'a, T> {
    row![
        row![
            space().width(Length::Fill),
            text_input("", "")
                .width(Length::Shrink)
                .style(|t, _| crate::iced_window::styles::invisible_text_input(t)),
            left
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
        space().width(10),
        row![
            right,
            text_input("", "")
                .width(Length::Shrink)
                .style(|t, _| crate::iced_window::styles::invisible_text_input(t))
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center)
    ]
    .into()
}

pub fn center_title<'a, T: Clone + 'a>(inner_text: &'a str) -> iced::Element<'a, T> {
    column![
        text(""),
        row![
            space().width(Length::Fill),
            text(inner_text).size(22).style(text::primary),
            space().width(Length::Fill),
        ]
    ]
    .into()
}