//! Built-in color themes (16-color ANSI palettes + foreground/background).

use gtk4::gdk::RGBA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    Flexoki,
    Monokai,
    SolarizedLight,
    SolarizedDark,
}

impl Theme {
    pub fn label(self) -> &'static str {
        match self {
            Theme::Flexoki => "Flexoki",
            Theme::Monokai => "Monokai",
            Theme::SolarizedLight => "Solarized Light",
            Theme::SolarizedDark => "Solarized Dark",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Flexoki => Palette {
                foreground: rgba("CECDC3"),
                background: rgba("1C1B1A"),
                colors: [
                    rgba("100F0F"), // black
                    rgba("AF3029"), // red
                    rgba("66800B"), // green
                    rgba("AD8301"), // yellow
                    rgba("205EA6"), // blue
                    rgba("A02F6F"), // magenta
                    rgba("24837B"), // cyan
                    rgba("CECDC3"), // white
                    rgba("575653"), // bright black
                    rgba("D14D41"), // bright red
                    rgba("879A39"), // bright green
                    rgba("D0A215"), // bright yellow
                    rgba("4385BE"), // bright blue
                    rgba("CE5D97"), // bright magenta
                    rgba("3AA99F"), // bright cyan
                    rgba("FFFCF0"), // bright white
                ],
            },
            Theme::Monokai => Palette {
                foreground: rgba("F8F8F2"),
                background: rgba("272822"),
                colors: [
                    rgba("272822"), // black
                    rgba("F92672"), // red
                    rgba("A6E22E"), // green
                    rgba("E6DB74"), // yellow
                    rgba("66D9EF"), // blue
                    rgba("AE81FF"), // magenta
                    rgba("A1EFE4"), // cyan
                    rgba("F8F8F2"), // white
                    rgba("75715E"), // bright black
                    rgba("FF6188"), // bright red
                    rgba("C4F56A"), // bright green
                    rgba("FFF0A5"), // bright yellow
                    rgba("8FE8FF"), // bright blue
                    rgba("C9AFFF"), // bright magenta
                    rgba("C4FFF5"), // bright cyan
                    rgba("FFFFFF"), // bright white
                ],
            },
            Theme::SolarizedDark => Palette {
                foreground: rgba("839496"),
                background: rgba("002B36"),
                colors: [
                    rgba("073642"), // black
                    rgba("DC322F"), // red
                    rgba("859900"), // green
                    rgba("B58900"), // yellow
                    rgba("268BD2"), // blue
                    rgba("D33682"), // magenta
                    rgba("2AA198"), // cyan
                    rgba("EEE8D5"), // white
                    rgba("002B36"), // bright black
                    rgba("CB4B16"), // bright red
                    rgba("586E75"), // bright green
                    rgba("657B83"), // bright yellow
                    rgba("839496"), // bright blue
                    rgba("6C71C4"), // bright magenta
                    rgba("93A1A1"), // bright cyan
                    rgba("FDF6E3"), // bright white
                ],
            },
            Theme::SolarizedLight => Palette {
                foreground: rgba("657B83"),
                background: rgba("FDF6E3"),
                colors: [
                    rgba("073642"), // black
                    rgba("DC322F"), // red
                    rgba("859900"), // green
                    rgba("B58900"), // yellow
                    rgba("268BD2"), // blue
                    rgba("D33682"), // magenta
                    rgba("2AA198"), // cyan
                    rgba("EEE8D5"), // white
                    rgba("002B36"), // bright black
                    rgba("CB4B16"), // bright red
                    rgba("586E75"), // bright green
                    rgba("657B83"), // bright yellow
                    rgba("839496"), // bright blue
                    rgba("6C71C4"), // bright magenta
                    rgba("93A1A1"), // bright cyan
                    rgba("FDF6E3"), // bright white
                ],
            },
        }
    }
}

pub struct Palette {
    pub foreground: RGBA,
    pub background: RGBA,
    pub colors: [RGBA; 16],
}

/// Parse a `#RRGGBB` hex string into an opaque RGBA color.
fn rgba(hex: &str) -> RGBA {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).expect("bad hex color");
    let g = u8::from_str_radix(&hex[2..4], 16).expect("bad hex color");
    let b = u8::from_str_radix(&hex[4..6], 16).expect("bad hex color");
    RGBA::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        1.0,
    )
}
