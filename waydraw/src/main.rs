use iced::Theme;

pub mod config;
pub mod p2p;
pub mod encoding;
pub mod mouse;
pub mod screen_capture;

mod window;

pub use config::Config;

fn main() {

    let arg = std::env::args().nth(1).unwrap_or("CatppuccinFrappe".to_owned());
    let theme = match arg.as_str() {
        "Light"                => Theme::Light,
        "Dark"                 => Theme::Dark,
        "Dracula"              => Theme::Dracula,
        "Nord"                 => Theme::Nord,
        "SolarizedLight"       => Theme::SolarizedLight,
        "SolarizedDark"        => Theme::SolarizedDark,
        "GruvboxLight"         => Theme::GruvboxLight,
        "GruvboxDark"          => Theme::GruvboxDark,
        "CatppuccinLatte"      => Theme::CatppuccinLatte,
        "CatppuccinFrappe"     => Theme::CatppuccinFrappe,
        "CatppuccinMacchiato"  => Theme::CatppuccinMacchiato,
        "CatppuccinMocha"      => Theme::CatppuccinMocha,
        "TokyoNight"           => Theme::TokyoNight,
        "TokyoNightStorm"      => Theme::TokyoNightStorm,
        "TokyoNightLight"      => Theme::TokyoNightLight,
        "KanagawaWave"         => Theme::KanagawaWave,
        "KanagawaDragon"       => Theme::KanagawaDragon,
        "KanagawaLotus"        => Theme::KanagawaLotus,
        "Moonfly"              => Theme::Moonfly,
        "Nightfly"             => Theme::Nightfly,
        "Oxocarbon"            => Theme::Oxocarbon,
        "Ferra"                => Theme::Ferra,
        _                      => Theme::CatppuccinFrappe
    };

    
    let _ = iced::application(
        window::Window::boot,
        window::Window::update,
        window::Window::view
    )
        .theme(theme)
        .subscription(window::window::subscription)
        .title("Waydraw")
 
        .run();
}