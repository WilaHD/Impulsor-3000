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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub theme_mode: ThemeMode,
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
        };
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed = toml::from_str::<AppConfig>(&toml).unwrap();

        assert!(toml.contains("theme_mode"));
        assert!(toml.contains("dark"));
        assert_eq!(parsed, config);
    }
}
