pub mod camera;
pub mod commands;

use camera::CameraState;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(CameraState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::list_cameras,
            commands::get_app_config,
            commands::set_recording_duration_sec,
            commands::connect_camera,
            commands::disconnect_camera,
            commands::get_settings,
            commands::get_setting_options,
            commands::set_iso,
            commands::set_aperture,
            commands::set_shutter_speed,
            commands::set_white_balance,
            commands::take_picture,
            commands::start_live_view,
            commands::stop_live_view,
            commands::get_live_view_frame,
            commands::list_folder_files,
            commands::read_file_chunk,
            commands::read_file_chunk_bytes,
            commands::upload_capture_file,
            commands::wait_for_file_stable,
            commands::upload_video_file,
            commands::upload_video_chunked,
            commands::upload_video_resumable,
            commands::list_printers,
            commands::print_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
