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
    find_library_path("PDFium", pdfium_library_relative_candidates()?)
}

pub fn lame_library_path() -> Result<PathBuf, String> {
    find_library_path("LAME", lame_library_relative_candidates()?)
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
        ("linux", "x86_64") => Ok(vec![
            PathBuf::from("libs/lame/linux-x64/libmp3lame.so"),
            PathBuf::from("libs/lame/linux-x64/libmp3lame.so.0"),
        ]),
        ("windows", "x86_64") => Ok(vec![PathBuf::from("libs/lame/win-x64/libmp3lame.dll")]),
        ("macos", "aarch64") => Ok(vec![
            PathBuf::from("libs/lame/mac-arm64/libmp3lame.0.dylib"),
            PathBuf::from("libs/lame/mac-arm64/libmp3lame.dylib"),
        ]),
        (os, arch) => Err(format!("Unsupported platform for LAME: {os}/{arch}")),
    }
}
