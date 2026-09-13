use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Deezer,
    SpotifyDark,
    NcmpcppBlue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "en")]
    En,
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::Fr => "Français",
            Language::En => "English",
        }
    }

    pub fn next(self) -> Language {
        match self {
            Language::Fr => Language::En,
            Language::En => Language::Fr,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color,
    pub accent_alt: Color,
    pub fg: Color,
    pub dim: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub status: Color,
}

impl Theme {
    pub fn colors(self) -> ThemeColors {
        match self {
            Theme::Deezer => ThemeColors {
                accent: Color::Rgb(0xA2, 0x38, 0xFF), // signal violet
                accent_alt: Color::Rgb(0xD0, 0x9A, 0xFF), // lilac wash
                fg: Color::Rgb(0xFD, 0xFC, 0xFE),     // paper white
                dim: Color::Rgb(0xA9, 0xA6, 0xAA),    // fog gray
                highlight_fg: Color::Rgb(0x19, 0x19, 0x22), // ink black
                highlight_bg: Color::Rgb(0xD0, 0x9A, 0xFF), // lilac wash
                status: Color::Rgb(0xA2, 0x38, 0xFF),
            },
            Theme::SpotifyDark => ThemeColors {
                accent: Color::Rgb(0x1D, 0xB9, 0x54),
                accent_alt: Color::Rgb(0x1E, 0xD7, 0x60),
                fg: Color::Rgb(0xFF, 0xFF, 0xFF),
                dim: Color::Rgb(0xB3, 0xB3, 0xB3),
                highlight_fg: Color::Rgb(0x1D, 0xB9, 0x54),
                highlight_bg: Color::Rgb(0x1F, 0x3D, 0x2A),
                status: Color::Rgb(0x1D, 0xB9, 0x54),
            },
            Theme::NcmpcppBlue => ThemeColors {
                accent: Color::Rgb(0x00, 0x87, 0xFF),
                accent_alt: Color::Rgb(0x00, 0xC8, 0xFF),
                fg: Color::Rgb(0xE4, 0xE4, 0xE4),
                dim: Color::Rgb(0x6C, 0x6C, 0x6C),
                highlight_fg: Color::Rgb(0x00, 0x87, 0xFF),
                highlight_bg: Color::Rgb(0x12, 0x3A, 0x5C),
                status: Color::Rgb(0xFF, 0xCC, 0x66),
            },
        }
    }

    pub fn next(self) -> Theme {
        match self {
            Theme::Deezer => Theme::SpotifyDark,
            Theme::SpotifyDark => Theme::NcmpcppBlue,
            Theme::NcmpcppBlue => Theme::Deezer,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioQuality {
    Kbps128,
    Kbps320,
    Flac,
}

impl AudioQuality {
    pub const fn format_code(self) -> u8 {
        match self {
            Self::Kbps128 => 1,
            Self::Kbps320 => 3,
            Self::Flac => 9,
        }
    }

    pub const fn from_format_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Kbps128),
            3 => Some(Self::Kbps320),
            9 => Some(Self::Flac),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub crossfade_enabled: bool,
    pub crossfade_duration_ms: u64,
    pub default_quality: AudioQuality,
    #[serde(default)]
    pub discord_rpc_enabled: bool,
    #[serde(default)]
    pub arl: String,
    #[serde(default)]
    pub language: Language,
    /// Whether the user has already picked a language during first-run setup.
    #[serde(default)]
    pub language_set: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Deezer,
            crossfade_enabled: false,
            crossfade_duration_ms: 0,
            default_quality: AudioQuality::Kbps320,
            discord_rpc_enabled: false,
            arl: String::new(),
            language: Language::default(),
            language_set: false,
        }
    }
}
