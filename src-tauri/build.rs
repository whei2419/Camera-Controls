fn main() {
    // Verify libgphoto2 is available via pkg-config.
    // Install with: brew install libgphoto2
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        pkg_config::probe_library("libgphoto2")
            .expect("libgphoto2 not found. Install it with: brew install libgphoto2");
    }

    tauri_build::build()
}
