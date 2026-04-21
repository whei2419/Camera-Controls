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

// ── File system helpers ───────────────────────────────────────────────────────

/// List image/video files in `folder`, sorted newest-first (by modified time).
/// `extensions` is e.g. ["jpg","jpeg","png","mp4","mkv"]
#[tauri::command]
pub async fn list_folder_files(
    folder: String,
    extensions: Vec<String>,
) -> Result<Vec<String>, String> {
    use std::fs;
    use std::time::SystemTime;

    let dir = fs::read_dir(&folder).map_err(|e| e.to_string())?;
    let exts: Vec<String> = extensions.iter().map(|e| e.to_lowercase()).collect();

    let mut entries: Vec<(SystemTime, String)> = dir
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !exts.contains(&ext) {
                return None;
            }
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((modified, path.to_string_lossy().into_owned()))
        })
        .collect();

    entries.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(entries.into_iter().map(|(_, p)| p).collect())
}

/// POST the capture file to the Laravel `upload-capture` route as multipart form-data.
/// `field_name` must match the key used in `$request->file(...)` on the server (often `image`).
#[tauri::command]
pub async fn upload_capture_file(
    file_path: String,
    url: String,
    field_name: String,
) -> Result<(), String> {
    use reqwest::multipart;
    use std::path::Path;
    use std::time::Duration;

    let path = Path::new(&file_path);
    let bytes = std::fs::read(path).map_err(|e| format!("read file: {e}"))?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("capture.jpg")
        .to_string();

    let mime = mime_for_capture_path(path);

    let part = multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str(mime)
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new().part(field_name, part);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(400).collect();
        return Err(format!("HTTP {status}: {snippet}"));
    }
    Ok(())
}

fn mime_for_capture_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("cr2") | Some("cr3") => "application/octet-stream",
        Some("nef") | Some("arw") => "application/octet-stream",
        _ => "application/octet-stream",
    }
}

// ── Printer helpers ───────────────────────────────────────────────────────────

/// List all installed printers on Windows via PowerShell.
#[tauri::command]
pub async fn list_printers() -> Result<Vec<String>, String> {
    use std::process::Command;
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object -ExpandProperty Name",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(printers)
}

/// Print an image file directly to the named printer (Windows only).
/// Uses mspaint /pt which bypasses the print dialog.
#[tauri::command]
pub async fn print_file(path: String, printer: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("mspaint")
        .args(["/pt", &path, &printer])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
