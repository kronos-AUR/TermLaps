//! Configuration loading: profiles (command + theme) and global settings.
//!
//! The configuration lives in `$XDG_CONFIG_HOME/termlaps/config.toml`
//! (usually `~/.config/termlaps/config.toml`). If it does not exist, a
//! default one is written there on first start.

use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

use crate::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Display name of the profile (tab title, menu entry).
    pub name: String,
    /// Command used to spawn the shell, e.g. `"/bin/bash"` or
    /// `["/usr/bin/env", "zsh", "-l"]`. The first entry is the executable.
    #[serde(deserialize_with = "de_string_or_seq")]
    pub command: Vec<String>,
    /// Color theme applied to every terminal of this profile.
    pub theme: Theme,
    /// Optional working directory for the spawned process.
    #[serde(default)]
    pub working_directory: Option<String>,
    /// Extra environment variables, as `"KEY=VALUE"` strings.
    #[serde(default)]
    pub env: Vec<String>,
    /// Optional font override for this profile (e.g. `"JetBrains Mono 11"`).
    #[serde(default)]
    pub font: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub scrollback_lines: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default = "default_profiles")]
    pub profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                font: Some("Monospace 12".to_string()),
                scrollback_lines: Some(10_000),
            },
            profiles: default_profiles(),
        }
    }
}

fn default_profiles() -> Vec<Profile> {
    let bash = || vec!["/bin/bash".to_string()];
    vec![
        Profile {
            name: "Flexoki".to_string(),
            command: bash(),
            theme: Theme::Flexoki,
            working_directory: None,
            env: Vec::new(),
            font: None,
        },
        Profile {
            name: "Monokai".to_string(),
            command: bash(),
            theme: Theme::Monokai,
            working_directory: None,
            env: Vec::new(),
            font: None,
        },
        Profile {
            name: "Solarized Light".to_string(),
            command: bash(),
            theme: Theme::SolarizedLight,
            working_directory: None,
            env: Vec::new(),
            font: None,
        },
        Profile {
            name: "Solarized Dark".to_string(),
            command: bash(),
            theme: Theme::SolarizedDark,
            working_directory: None,
            env: Vec::new(),
            font: None,
        },
    ]
}

fn de_string_or_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrSeq {
        One(String),
        Many(Vec<String>),
    }

    Ok(match StringOrSeq::deserialize(deserializer)? {
        StringOrSeq::One(s) => vec![s],
        StringOrSeq::Many(v) => v,
    })
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("termlaps").join("config.toml")
}

/// Load the configuration, writing a default config file if none exists.
pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("termlaps: invalid config at {path:?}: {err}\nUsing defaults.");
                Config::default()
            }
        },
        Err(_) => {
            let config = Config::default();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match toml::to_string_pretty(&config) {
                Ok(text) => {
                    if std::fs::write(&path, text).is_ok() {
                        eprintln!("termlaps: default config written to {path:?}");
                    }
                }
                Err(err) => eprintln!("termlaps: could not serialize default config: {err}"),
            }
            config
        }
    }
}
