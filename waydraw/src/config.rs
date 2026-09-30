use std::{collections::HashMap, path::PathBuf};

use iroh::SecretKey;

pub struct Config {
    pub secret_key: SecretKey,
    pub known_hosts: HashMap<String, String>,
    pub theme: iced::Theme
}

impl Config {
    pub fn load_or_generate() -> Self {
        // let config_path = config_path();
        let secret_key = load_or_create_secret_key().unwrap();
        let known_hosts = load_hosts();
        let theme = load_theme();

        Self {
            secret_key,
            known_hosts,
            theme
        }
    }

    pub fn write(&self) {
        write_hosts(&self.known_hosts);
    }
}

fn config_path() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .expect("APPDATA not set");
    
    #[cfg(target_os = "linux")]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config"));

    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("No home directory found")
        .join("Library/Application Support");
    
    base.join("waydraw")
}

pub fn load_or_create_secret_key() -> Result<SecretKey, std::io::Error> {
    let path = config_path().join("secret_key");

    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return Ok(SecretKey::from_bytes(&bytes));
        }
    }

    let key = SecretKey::generate();
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, key.to_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(key)
}

fn load_hosts() -> HashMap<String, String> {
    let path = config_path().join("known_hosts");

    let serialized = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            let _ = std::fs::write(&path, "");
            String::from("")
        }
    };

    let map: HashMap<String, String> = serde_json::from_str(&serialized).unwrap_or([].into());
    return map;
}

fn write_hosts(map: &HashMap<String, String>) {
    let path = config_path().join("known_hosts");

    let serialized = serde_json::to_string(map).unwrap();

    std::fs::write(path, serialized).unwrap();
}

fn load_theme() -> iced::Theme {
    let path = config_path().join("theme");

    let theme = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return iced::Theme::CatppuccinFrappe
    };
    
    match theme.trim() {
        "Light"                => iced::Theme::Light,
        "Dark"                 => iced::Theme::Dark,
        "Dracula"              => iced::Theme::Dracula,
        "Nord"                 => iced::Theme::Nord,
        "SolarizedLight"       => iced::Theme::SolarizedLight,
        "SolarizedDark"        => iced::Theme::SolarizedDark,
        "GruvboxLight"         => iced::Theme::GruvboxLight,
        "GruvboxDark"          => iced::Theme::GruvboxDark,
        "CatppuccinLatte"      => iced::Theme::CatppuccinLatte,
        "CatppuccinFrappe"     => iced::Theme::CatppuccinFrappe,
        "CatppuccinMacchiato"  => iced::Theme::CatppuccinMacchiato,
        "CatppuccinMocha"      => iced::Theme::CatppuccinMocha,
        "TokyoNight"           => iced::Theme::TokyoNight,
        "TokyoNightStorm"      => iced::Theme::TokyoNightStorm,
        "TokyoNightLight"      => iced::Theme::TokyoNightLight,
        "KanagawaWave"         => iced::Theme::KanagawaWave,
        "KanagawaDragon"       => iced::Theme::KanagawaDragon,
        "KanagawaLotus"        => iced::Theme::KanagawaLotus,
        "Moonfly"              => iced::Theme::Moonfly,
        "Nightfly"             => iced::Theme::Nightfly,
        "Oxocarbon"            => iced::Theme::Oxocarbon,
        "Ferra"                => iced::Theme::Ferra,
        _                      => iced::Theme::CatppuccinFrappe
    }
}