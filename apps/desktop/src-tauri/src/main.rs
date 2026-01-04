#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Force X11 backend on Linux to avoid WebKitGTK Wayland protocol errors
    #[cfg(target_os = "linux")]
    std::env::set_var("GDK_BACKEND", "x11");

    app_lib::run();
}
