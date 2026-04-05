#[cfg(windows)]
extern crate winres;

use std::path::PathBuf;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
        .join(relative)
}

#[cfg(windows)]
fn main() {
    let lame_dir = manifest_path("libs/lame/win-x64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=static=mp3lame");
    println!("cargo:rerun-if-changed=libs/lame/win-x64/mp3lame.lib");
    println!("cargo:rerun-if-changed=libs/lame/win-x64/libmp3lame.dll");

    let mut res = winres::WindowsResource::new();
    res.set_icon("imgs/icon.ico");
    res.compile().unwrap();
}

#[cfg(target_os = "macos")]
fn main() {
    let lame_dir = manifest_path("libs/lame/mac-arm64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rerun-if-changed=libs/lame/mac-arm64/libmp3lame.dylib");
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    let lame_dir = manifest_path("libs/lame/linux-x64");

    println!("cargo:rustc-link-search=native={}", lame_dir.display());
    println!("cargo:rustc-link-lib=dylib=mp3lame");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/libs/lame/linux-x64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../libs/lame/linux-x64");
    println!("cargo:rerun-if-changed=libs/lame/linux-x64/libmp3lame.so");
}
