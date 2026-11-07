use std::{env, ffi::OsString, path::PathBuf};

pub const APP_SETTINGS_FILE_ENV_VAR: &str = "IMPULSOR3000_SETTINGS_FILE";

pub fn app_settings_file_path() -> Result<PathBuf, String> {
    app_settings_file_path_from(env::var_os(APP_SETTINGS_FILE_ENV_VAR), dirs::config_dir())
}

fn app_settings_file_path_from(
    settings_file_override: Option<OsString>,
    config_dir: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(settings_file_override) = settings_file_override {
        if !settings_file_override.is_empty() {
            return Ok(PathBuf::from(settings_file_override));
        }
    }

    let config_dir =
        config_dir.ok_or_else(|| String::from("Could not determine OS configuration directory"))?;

    Ok(config_dir.join("impulsor3000").join("settings.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_file_path_uses_env_override() {
        let override_path = PathBuf::from("custom-settings.toml");
        let actual = app_settings_file_path_from(
            Some(override_path.clone().into_os_string()),
            Some(PathBuf::from("config")),
        )
        .unwrap();

        assert_eq!(actual, override_path);
    }

    #[test]
    fn settings_file_path_ignores_empty_env_override() {
        let actual =
            app_settings_file_path_from(Some(OsString::new()), Some(PathBuf::from("config")))
                .unwrap();

        assert_eq!(
            actual,
            PathBuf::from("config")
                .join("impulsor3000")
                .join("settings.toml")
        );
    }
}
