#[cfg(windows)]
extern crate winres;

use std::path::PathBuf;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
        .join(relative)
}

fn app_version() -> String {
    println!("cargo:rerun-if-env-changed=IMPULSOR_BUILD_VERSION");

    std::env::var("IMPULSOR_BUILD_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION not set")
}

fn export_app_version() {
    println!("cargo:rustc-env=IMPULSOR_APP_VERSION={}", app_version());
}

#[cfg(windows)]
fn main() {
    let app_version = app_version();
    println!("cargo:rustc-env=IMPULSOR_APP_VERSION={app_version}");

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
    export_app_version();

    let lame_dir = manifest_path("libs/lame/mac-arm64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rerun-if-changed=libs/lame/mac-arm64/libmp3lame.dylib");
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    export_app_version();

    let lame_dir = manifest_path("libs/lame/linux-x64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/libs/lame/linux-x64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../libs/lame/linux-x64");
    println!("cargo:rerun-if-changed=libs/lame/linux-x64/libmp3lame.so");
}
