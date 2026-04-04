use std::{
    env,
    path::{Path, PathBuf},
};

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
    Ok(resource_root()?.join(pdfium_library_relative_path()?))
}

pub fn lame_library_path() -> Result<PathBuf, String> {
    Ok(resource_root()?.join(lame_library_relative_path()?))
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

fn pdfium_library_relative_path() -> Result<PathBuf, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(PathBuf::from("libs/pdfium/linux-x64/libpdfium.so")),
        ("windows", "x86_64") => Ok(PathBuf::from("libs/pdfium/win-x64/pdfium.dll")),
        ("macos", "aarch64") => Ok(PathBuf::from("libs/pdfium/mac-x64-arm/libpdfium.dylib")),
        (os, arch) => Err(format!("Unsupported platform for PDFium: {os}/{arch}")),
    }
}

fn lame_library_relative_path() -> Result<PathBuf, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(PathBuf::from("libs/lame/linux-x64/libmp3lame.so")),
        ("windows", "x86_64") => Ok(PathBuf::from("libs/lame/win-x64/libmp3lame.dll")),
        ("macos", "aarch64") => Ok(PathBuf::from("libs/lame/mac-x64-arm/libmp3lame.dylib")),
        (os, arch) => Err(format!("Unsupported platform for LAME: {os}/{arch}")),
    }
}
