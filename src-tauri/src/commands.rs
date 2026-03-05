// Tauri commands – exposed to the Vue frontend via invoke()

use tauri::State;
use tokio::sync::Mutex;

use crate::camera::{self, CameraInfo, CameraState, SettingOptions, ShootingSettings};

pub type AppState = Mutex<CameraState>;

// ── Camera discovery ──────────────────────────────────────────────────────────
#[tauri::command]
pub async fn list_cameras(state: State<'_, AppState>) -> Result<Vec<CameraInfo>, String> {
    let s = state.lock().await;
    camera::list_cameras(&s).await.map_err(|e| e.to_string())
}

// ── Connection ────────────────────────────────────────────────────────────────
#[tauri::command]
pub async fn connect_camera(
    state: State<'_, AppState>,
    index: usize,
) -> Result<CameraInfo, String> {
    let mut s = state.lock().await;
    camera::connect(&mut s, index)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_camera(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.lock().await;
    camera::disconnect(&mut s);
    Ok(())
}

// ── Settings ──────────────────────────────────────────────────────────────────
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<ShootingSettings, String> {
    let s = state.lock().await;
    camera::get_settings(&s).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting_options(state: State<'_, AppState>) -> Result<SettingOptions, String> {
    let s = state.lock().await;
    camera::get_setting_options(&s)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_iso(state: State<'_, AppState>, value: String) -> Result<(), String> {
    let s = state.lock().await;
    camera::set_config(&s, "iso", &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_aperture(state: State<'_, AppState>, value: String) -> Result<(), String> {
    let s = state.lock().await;
    camera::set_config(&s, "aperture", &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_shutter_speed(state: State<'_, AppState>, value: String) -> Result<(), String> {
    let s = state.lock().await;
    camera::set_config(&s, "shutterspeed", &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_white_balance(state: State<'_, AppState>, value: String) -> Result<(), String> {
    let s = state.lock().await;
    camera::set_config(&s, "whitebalance", &value)
        .await
        .map_err(|e| e.to_string())
}

// ── Shutter ───────────────────────────────────────────────────────────────────
#[tauri::command]
pub async fn take_picture(state: State<'_, AppState>) -> Result<(), String> {
    let s = state.lock().await;
    camera::take_picture(&s).await.map_err(|e| e.to_string())
}

// ── Live view ─────────────────────────────────────────────────────────────────
#[tauri::command]
pub async fn start_live_view(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.lock().await;
    camera::start_live_view(&mut s)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_live_view(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.lock().await;
    camera::stop_live_view(&mut s)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_view_frame(state: State<'_, AppState>) -> Result<String, String> {
    let s = state.lock().await;
    camera::capture_live_view_frame(&s)
        .await
        .map_err(|e| e.to_string())
}
