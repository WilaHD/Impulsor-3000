#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    println!("cargo:rustc-link-search=native=libs/lame/windows-x64");
    println!("cargo:rustc-link-lib=static=mp3lame");

    let mut res = winres::WindowsResource::new();
    res.set_icon("imgs/icon.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {
}