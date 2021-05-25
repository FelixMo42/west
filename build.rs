fn main() {
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=xkbcommon");
        println!("cargo:rustc-link-lib=wayland-client");
        println!("cargo:rustc-link-lib=wayland-cursor");
        println!("cargo:rustc-link-lib=wayland-egl");
    }
}
