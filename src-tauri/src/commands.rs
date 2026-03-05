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

pub type AppState = Mutex<CameraState>;

// ── Helper ─────────────────────────────────────────────────────────────────────
fn map_err(e: CameraError) -> String {
    e.to_string()
}

// ── SDK init / teardown ───────────────────────────────────────────────────────
#[tauri::command]
pub fn init_sdk(state: State<AppState>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::init_sdk(&mut s).map_err(map_err)
}

#[tauri::command]
pub fn terminate_sdk(state: State<AppState>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::terminate_sdk(&mut s).map_err(map_err)
}

// ── Camera discovery ──────────────────────────────────────────────────────────
#[tauri::command]
pub fn list_cameras(state: State<AppState>) -> Result<Vec<CameraInfo>, String> {
    let mut s = state.lock().unwrap();
    // Auto-init SDK if needed
    if !s.sdk_initialised {
        edsdk::init_sdk(&mut s).map_err(map_err)?;
    }
    edsdk::list_cameras(&s).map_err(map_err)
}

// ── Connection ────────────────────────────────────────────────────────────────
#[tauri::command]
pub fn connect_camera(state: State<AppState>, index: usize) -> Result<CameraInfo, String> {
    let mut s = state.lock().unwrap();
    edsdk::connect_camera(&mut s, index).map_err(map_err)
}

#[tauri::command]
pub fn disconnect_camera(state: State<AppState>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::disconnect_camera(&mut s).map_err(map_err)
}

// ── Settings read ─────────────────────────────────────────────────────────────
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<ShootingSettings, String> {
    let s = state.lock().unwrap();
    edsdk::get_settings(&s).map_err(map_err)
}

// ── Settings write ────────────────────────────────────────────────────────────
#[tauri::command]
pub fn set_av(state: State<AppState>, value: u32) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::set_av(&mut s, value).map_err(map_err)
}

#[tauri::command]
pub fn set_tv(state: State<AppState>, value: u32) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::set_tv(&mut s, value).map_err(map_err)
}

#[tauri::command]
pub fn set_iso(state: State<AppState>, value: u32) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::set_iso(&mut s, value).map_err(map_err)
}

#[tauri::command]
pub fn set_white_balance(state: State<AppState>, value: i32) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::set_white_balance(&mut s, value).map_err(map_err)
}

// ── Shutter ───────────────────────────────────────────────────────────────────
#[tauri::command]
pub fn take_picture(state: State<AppState>) -> Result<(), String> {
    let s = state.lock().unwrap();
    edsdk::take_picture(&s).map_err(map_err)
}

#[tauri::command]
pub fn press_shutter_halfway(state: State<AppState>) -> Result<(), String> {
    let s = state.lock().unwrap();
    edsdk::press_shutter_halfway(&s).map_err(map_err)
}

#[tauri::command]
pub fn release_shutter(state: State<AppState>) -> Result<(), String> {
    let s = state.lock().unwrap();
    edsdk::release_shutter(&s).map_err(map_err)
}

// ── Live view ─────────────────────────────────────────────────────────────────
#[tauri::command]
pub fn start_live_view(state: State<AppState>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::start_live_view(&mut s).map_err(map_err)
}

#[tauri::command]
pub fn stop_live_view(state: State<AppState>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    edsdk::stop_live_view(&mut s).map_err(map_err)
}

#[tauri::command]
pub fn get_live_view_frame(state: State<AppState>) -> Result<String, String> {
    let s = state.lock().unwrap();
    edsdk::capture_live_view_frame(&s).map_err(map_err)
}

// ── Property option lists (for dropdowns) ─────────────────────────────────────
#[derive(Serialize, Deserialize, Debug)]
pub struct PropOption<T> {
    pub value: T,
    pub label: String,
}

#[tauri::command]
pub fn get_av_options() -> Vec<PropOption<u32>> {
    properties::av_options()
        .into_iter()
        .map(|(v, l)| PropOption { value: v, label: l })
        .collect()
}

#[tauri::command]
pub fn get_tv_options() -> Vec<PropOption<u32>> {
    properties::tv_options()
        .into_iter()
        .map(|(v, l)| PropOption { value: v, label: l })
        .collect()
}

#[tauri::command]
pub fn get_iso_options() -> Vec<PropOption<u32>> {
    properties::iso_options()
        .into_iter()
        .map(|(v, l)| PropOption { value: v, label: l })
        .collect()
}

#[tauri::command]
pub fn get_wb_options() -> Vec<PropOption<i32>> {
    properties::wb_options()
        .into_iter()
        .map(|(v, l)| PropOption { value: v, label: l })
        .collect()
}
