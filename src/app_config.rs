use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::platform_paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Auto,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::Auto, ThemeMode::Light, ThemeMode::Dark];

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Auto => "Automatisch",
            ThemeMode::Light => "Hell",
            ThemeMode::Dark => "Dunkel",
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Auto
    }
}

pub const DEFAULT_LINE_SPACING_PERCENT: u16 = 133;
pub const MIN_LINE_SPACING_PERCENT: u16 = 80;
pub const MAX_LINE_SPACING_PERCENT: u16 = 250;

fn default_line_spacing_percent() -> u16 {
    DEFAULT_LINE_SPACING_PERCENT
}

fn deserialize_line_spacing<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u16, D::Error> {
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value
        .as_integer()
        .and_then(|value| u16::try_from(value).ok())
        .filter(|value| (MIN_LINE_SPACING_PERCENT..=MAX_LINE_SPACING_PERCENT).contains(value))
        .unwrap_or(DEFAULT_LINE_SPACING_PERCENT))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(
        default = "default_line_spacing_percent",
        deserialize_with = "deserialize_line_spacing"
    )]
    pub line_spacing_percent: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::default(),
            line_spacing_percent: DEFAULT_LINE_SPACING_PERCENT,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        Self::try_load().unwrap_or_default()
    }

    pub fn try_load() -> Result<Self, String> {
        let settings_path = platform_paths::app_settings_file_path()?;

        match fs::read_to_string(&settings_path) {
            Ok(settings) => toml::from_str(&settings).map_err(|e| {
                format!(
                    "Failed to parse settings file {}: {e}",
                    settings_path.display()
                )
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!(
                "Failed to read settings file {}: {error}",
                settings_path.display()
            )),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let settings_path = platform_paths::app_settings_file_path()?;
        self.save_to_path(&settings_path)
    }

    fn save_to_path(&self, settings_path: &Path) -> Result<(), String> {
        if let Some(settings_dir) = settings_path.parent() {
            fs::create_dir_all(settings_dir).map_err(|error| {
                format!(
                    "Failed to create settings directory {}: {error}",
                    settings_dir.display()
                )
            })?;
        }

        let settings = toml::to_string_pretty(self)
            .map_err(|error| format!("Failed to write settings TOML: {error}"))?;

        fs::write(settings_path, settings).map_err(|error| {
            format!(
                "Failed to write settings file {}: {error}",
                settings_path.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_auto_theme_mode() {
        let config = toml::from_str::<AppConfig>("").unwrap();

        assert_eq!(config.theme_mode, ThemeMode::Auto);
    }

    #[test]
    fn serializes_theme_mode_as_lowercase_toml_value() {
        let config = AppConfig {
            theme_mode: ThemeMode::Dark,
            ..AppConfig::default()
        };
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed = toml::from_str::<AppConfig>(&toml).unwrap();

        assert!(toml.contains("theme_mode"));
        assert!(toml.contains("dark"));
        assert_eq!(parsed, config);
    }

    #[test]
    fn line_spacing_is_backwards_compatible_and_round_trips() {
        let old = toml::from_str::<AppConfig>("theme_mode = 'dark'").unwrap();
        assert_eq!(old.line_spacing_percent, 133);
        assert_eq!(old.theme_mode, ThemeMode::Dark);
        let configured = AppConfig {
            line_spacing_percent: 175,
            ..old
        };
        let path =
            std::env::temp_dir().join(format!("impulsor-settings-{}.toml", std::process::id()));
        configured.save_to_path(&path).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert_eq!(toml::from_str::<AppConfig>(&saved).unwrap(), configured);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_line_spacing_preserves_other_settings() {
        for value in ["-1", "0", "251", "'falsch'"] {
            let config: AppConfig = toml::from_str(&format!(
                "theme_mode = 'dark'\nline_spacing_percent = {value}"
            ))
            .unwrap();
            assert_eq!(config.theme_mode, ThemeMode::Dark);
            assert_eq!(config.line_spacing_percent, DEFAULT_LINE_SPACING_PERCENT);
        }
    }
}
