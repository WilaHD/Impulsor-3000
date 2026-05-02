use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

pub const APP_SETTINGS_FILE_ENV_VAR: &str = "IMPULSOR3000_SETTINGS_FILE";

pub fn resource_root() -> Result<PathBuf, String> {
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to read current executable path: {e}"))?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        format!(
            "Executable path has no parent directory: {}",
            exe_path.display()
        )
    })?;

    for candidate in candidate_resource_roots(exe_dir) {
        if candidate.join("libs").is_dir() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Could not find a resource root containing 'libs' from executable {}",
        exe_path.display()
    ))
}

pub fn pdfium_library_path() -> Result<PathBuf, String> {
    find_library_path("PDFium", pdfium_library_relative_candidates()?)
}

pub fn lame_library_path() -> Result<PathBuf, String> {
    find_library_path("LAME", lame_library_relative_candidates()?)
}

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

fn candidate_resource_roots(exe_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(resources_dir) = macos_bundle_resources_dir(exe_dir) {
        candidates.push(resources_dir);
    }

    candidates.push(exe_dir.to_path_buf());
    candidates.extend(exe_dir.ancestors().skip(1).map(Path::to_path_buf));

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir);
    }

    candidates
}

fn macos_bundle_resources_dir(exe_dir: &Path) -> Option<PathBuf> {
    let macos_dir = exe_dir.file_name()?;
    if macos_dir != "MacOS" {
        return None;
    }

    let contents_dir = exe_dir.parent()?;
    if contents_dir.file_name()? != "Contents" {
        return None;
    }

    Some(contents_dir.join("Resources"))
}

fn find_library_path(name: &str, candidates: Vec<PathBuf>) -> Result<PathBuf, String> {
    let root = resource_root()?;
    let absolute_candidates = candidates
        .into_iter()
        .map(|candidate| root.join(candidate))
        .collect::<Vec<_>>();

    if let Some(path) = absolute_candidates.iter().find(|path| path.is_file()) {
        return Ok(path.clone());
    }

    let tried = absolute_candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Err(format!("{name} library not found. Tried: {tried}"))
}

fn pdfium_library_relative_candidates() -> Result<Vec<PathBuf>, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(vec![PathBuf::from("libs/pdfium/linux-x64/libpdfium.so")]),
        ("windows", "x86_64") => Ok(vec![PathBuf::from("libs/pdfium/win-x64/pdfium.dll")]),
        ("macos", "aarch64") => Ok(vec![PathBuf::from("libs/pdfium/mac-arm64/libpdfium.dylib")]),
        (os, arch) => Err(format!("Unsupported platform for PDFium: {os}/{arch}")),
    }
}

fn lame_library_relative_candidates() -> Result<Vec<PathBuf>, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(vec![PathBuf::from("libs/lame/linux-x64/libmp3lame.so")]),
        ("windows", "x86_64") => Ok(vec![PathBuf::from("libs/lame/win-x64/libmp3lame.dll")]),
        ("macos", "aarch64") => Ok(vec![PathBuf::from("libs/lame/mac-arm64/libmp3lame.dylib")]),
        (os, arch) => Err(format!("Unsupported platform for LAME: {os}/{arch}")),
    }
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
