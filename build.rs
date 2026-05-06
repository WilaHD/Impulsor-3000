#[cfg(windows)]
extern crate winres;

use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
}

fn manifest_path(relative: &str) -> PathBuf {
    manifest_dir().join(relative)
}

fn app_version() -> String {
    println!("cargo:rerun-if-env-changed=IMPULSOR_BUILD_VERSION");

    std::env::var("IMPULSOR_BUILD_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION not set")
}

fn git_tag() -> String {
    println!("cargo:rerun-if-env-changed=IMPULSOR_GIT_TAG");

    if let Ok(tag) = std::env::var("IMPULSOR_GIT_TAG") {
        if !tag.is_empty() {
            return tag;
        }
    }

    git_output(["describe", "--tags", "--exact-match", "HEAD"]).unwrap_or_default()
}

fn app_version_display(app_version: &str) -> String {
    let exact_tag = git_tag();
    if !exact_tag.is_empty() {
        return exact_tag;
    }

    let latest_known_version = git_output(["describe", "--tags", "--abbrev=0", "HEAD"])
        .unwrap_or_else(|| format!("v{app_version}"));

    match git_output(["rev-parse", "--short=12", "HEAD"]) {
        Some(commit_sha) => format!("{latest_known_version} ({commit_sha})"),
        None => latest_known_version,
    }
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    track_git_metadata_inputs();

    std::process::Command::new("git")
        .args(args)
        .current_dir(manifest_dir())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|tag| tag.trim().to_string())
        .filter(|output| !output.is_empty())
}

fn track_git_metadata_inputs() {
    let git_dir = manifest_path(".git");

    if git_dir.is_dir() {
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join("packed-refs").display()
        );
        println!("cargo:rerun-if-changed={}", git_dir.join("refs").display());
    }
}

fn export_build_metadata(app_version: &str) {
    println!("cargo:rustc-env=IMPULSOR_APP_VERSION={app_version}");
    println!("cargo:rustc-env=IMPULSOR_GIT_TAG={}", git_tag());
    println!(
        "cargo:rustc-env=IMPULSOR_APP_VERSION_DISPLAY={}",
        app_version_display(app_version)
    );
}

#[cfg(windows)]
fn main() {
    let app_version = app_version();
    export_build_metadata(&app_version);

    let lame_dir = manifest_path("libs/lame/win-x64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=static=mp3lame");
    println!("cargo:rerun-if-changed=libs/lame/win-x64/mp3lame.lib");
    println!("cargo:rerun-if-changed=libs/lame/win-x64/libmp3lame.dll");

    let mut res = winres::WindowsResource::new();
    res.set_icon("imgs/icon.ico");
    res.set("FileVersion", &app_version);
    res.set("ProductVersion", &app_version);
    res.set_version_info(
        winres::VersionInfo::FILEVERSION,
        windows_numeric_version(&app_version),
    );
    res.set_version_info(
        winres::VersionInfo::PRODUCTVERSION,
        windows_numeric_version(&app_version),
    );
    res.compile().unwrap();
}

#[cfg(windows)]
fn windows_numeric_version(version: &str) -> u64 {
    let base_version = version
        .split(|char| char == '-' || char == '+')
        .next()
        .unwrap_or(version);
    let mut parts = base_version
        .split('.')
        .map(|part| part.parse::<u16>().unwrap_or_default());

    let major = parts.next().unwrap_or_default() as u64;
    let minor = parts.next().unwrap_or_default() as u64;
    let patch = parts.next().unwrap_or_default() as u64;

    (major << 48) | (minor << 32) | (patch << 16)
}

#[cfg(target_os = "macos")]
fn main() {
    let app_version = app_version();
    export_build_metadata(&app_version);

    let lame_dir = manifest_path("libs/lame/mac-arm64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rerun-if-changed=libs/lame/mac-arm64/libmp3lame.dylib");
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    let app_version = app_version();
    export_build_metadata(&app_version);

    let lame_dir = manifest_path("libs/lame/linux-x64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/libs/lame/linux-x64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../libs/lame/linux-x64");
    println!("cargo:rerun-if-changed=libs/lame/linux-x64/libmp3lame.so");
}
