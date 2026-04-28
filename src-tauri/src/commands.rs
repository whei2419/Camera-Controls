// Tauri commands – exposed to the Vue frontend via invoke()

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tokio::sync::Mutex;

use crate::camera::{self, CameraInfo, CameraState, SettingOptions, ShootingSettings};

pub type AppState = Mutex<CameraState>;

const APP_CONFIG_FILE: &str = "app-config.json";
const RECORDING_DURATION_MIN_SEC: u64 = 3;
const RECORDING_DURATION_MAX_SEC: u64 = 600;
const RECORDING_DURATION_DEFAULT_SEC: u64 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub recording_duration_sec: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            recording_duration_sec: RECORDING_DURATION_DEFAULT_SEC,
        }
    }
}

fn clamp_recording_duration(seconds: u64) -> u64 {
    seconds.clamp(RECORDING_DURATION_MIN_SEC, RECORDING_DURATION_MAX_SEC)
}

fn config_file_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let mut dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    dir.push(APP_CONFIG_FILE);
    Ok(dir)
}

fn read_app_config(app: &tauri::AppHandle) -> Result<AppConfig, String> {
    use std::io::ErrorKind;

    let path = config_file_path(app)?;
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(AppConfig::default()),
        Err(e) => return Err(e.to_string()),
    };

    let mut config: AppConfig = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    config.recording_duration_sec = clamp_recording_duration(config.recording_duration_sec);
    Ok(config)
}

fn write_app_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_file_path(app)?;
    let payload = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, payload).map_err(|e| e.to_string())
}

// ── App config ───────────────────────────────────────────────────────────────
#[tauri::command]
pub async fn get_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    read_app_config(&app)
}

#[tauri::command]
pub async fn set_recording_duration_sec(
    app: tauri::AppHandle,
    seconds: u64,
) -> Result<AppConfig, String> {
    let mut config = read_app_config(&app)?;
    config.recording_duration_sec = clamp_recording_duration(seconds);
    write_app_config(&app, &config)?;
    Ok(config)
}

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
    since_ms: Option<u64>,
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
            if let Some(since) = since_ms {
                let ms = modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .ok()?
                    .as_millis() as u64;
                if ms < since {
                    return None;
                }
            }
            Some((modified, path.to_string_lossy().into_owned()))
        })
        .collect();

    entries.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(entries.into_iter().map(|(_, p)| p).collect())
}

/// POST the capture file to the Laravel `upload-capture` route as multipart form-data.
/// POST a photo/capture file to the Laravel `upload-capture` route as a base64 data URI
/// in a JSON body: `{"image": "data:image/jpeg;base64,..."}`.
///
/// The server's CaptureUploadController already handles this path
/// (`$request->filled('image')` branch). Sending JSON avoids all multipart/form-data
/// parsing on Apache/PHP-FPM, which was causing UPLOAD_ERR_PARTIAL regardless of
/// Content-Length, HTTP version, or TLS backend.
#[tauri::command]
pub async fn upload_capture_file(
    file_path: String,
    url: String,
    _field_name: String, // kept for API compatibility; server always reads key "image"
) -> Result<(), String> {
    use std::path::Path;
    use std::time::Duration;

    let path = Path::new(&file_path);
    let file_bytes = std::fs::read(path).map_err(|e| format!("read file: {e}"))?;
    let mime = mime_for_capture_path(path);

    eprintln!(
        "[upload_capture_file] path={file_path:?} mime={mime} bytes={} url={url}",
        file_bytes.len()
    );

    // Use multipart/form-data - more reliable for file uploads
    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        )
        .mime_str(&mime)
        .map_err(|e| e.to_string())?;

    let form = reqwest::multipart::Form::new().part("image", file_part);

    eprintln!("[upload_capture_file] Sending multipart form-data with field 'image'");

    let client = reqwest::Client::builder()
        .use_native_tls()
        // Try without forcing HTTP/1.1 to avoid UPLOAD_ERR_PARTIAL
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .post(&url)
        .header("Accept", "application/json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    eprintln!("[upload_capture_file] status={status}");

    if status.is_success() {
        eprintln!("[upload_capture_file] SUCCESS — file uploaded ok");
        return Ok(());
    }

    let status = res.status();
    let body_txt = res.text().await.unwrap_or_default();
    eprintln!("[upload_capture_file] error body={body_txt:?}");
    Err(format!(
        "HTTP {status}: {}",
        &body_txt[..body_txt.len().min(400)]
    ))
}

/// Wait until a file's size stops changing (i.e. OBS has finished flushing it).
/// Polls every 500 ms; considers the file stable after `stable_checks` consecutive
/// reads with the same size AND modified time. Times out after `timeout_secs`
/// and returns an error to prevent uploading a partial/corrupted file.
#[tauri::command]
pub async fn wait_for_file_stable(
    file_path: String,
    timeout_secs: Option<u64>,
    stable_checks: Option<u32>,
) -> Result<u64, String> {
    use std::path::Path;
    use std::time::SystemTime;
    use tokio::time::{sleep, Duration};

    let path = Path::new(&file_path);
    let max_secs = timeout_secs.unwrap_or(30);
    let needed = stable_checks.unwrap_or(3);
    let poll_ms = 500u64;
    let max_polls = (max_secs * 1000 / poll_ms) as u32;

    let mut prev_size: u64 = 0;
    let mut prev_mtime: Option<SystemTime> = None;
    let mut stable_count: u32 = 0;

    for _ in 0..max_polls {
        sleep(Duration::from_millis(poll_ms)).await;
        let meta = match tokio::fs::metadata(path).await {
            Ok(m) => m,
            Err(_) => {
                stable_count = 0;
                prev_size = 0;
                prev_mtime = None;
                continue;
            }
        };

        let size = meta.len();
        let mtime = meta.modified().ok();

        if size > 0 && size == prev_size && mtime.is_some() && mtime == prev_mtime {
            stable_count += 1;
            if stable_count >= needed {
                return Ok(size);
            }
        } else {
            stable_count = 0;
            prev_size = size;
            prev_mtime = mtime;
        }
    }

    Err(format!(
        "file did not stabilize within {}s (last size: {})",
        max_secs, prev_size
    ))
}

/// Upload a video file in 5 MB chunks to avoid timeouts on large files.
/// Calls `url_chunk` once per chunk, then `url_assemble` to stitch them on the server.
#[tauri::command]
pub async fn upload_video_chunked(
    file_path: String,
    url_chunk: String,
    url_assemble: String,
) -> Result<serde_json::Value, String> {
    use reqwest::multipart;
    use std::path::Path;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::io::AsyncReadExt;

    const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5 MB per chunk

    let path = Path::new(&file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();

    let file_size = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("stat: {e}"))?
        .len() as usize;

    let total_chunks = (file_size + CHUNK_SIZE - 1).max(1) / CHUNK_SIZE;

    // Unique upload ID: safe charset only (matches Laravel /^[a-zA-Z0-9_\-]+$/)
    // Avoid deriving from filename because spaces/parentheses can break validation.
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let upload_id = format!("vid_{ts}");

    // Client with per-chunk timeout (2 min is plenty for 5 MB)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| format!("open: {e}"))?;

    for chunk_index in 0..total_chunks {
        let mut buf = vec![0u8; CHUNK_SIZE];
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| format!("read chunk {chunk_index}: {e}"))?;
        buf.truncate(n);
        if n == 0 {
            break;
        }

        let part = multipart::Part::bytes(buf)
            .file_name(filename.clone())
            .mime_str("application/octet-stream")
            .map_err(|e| e.to_string())?;

        let form = multipart::Form::new()
            .text("upload_id", upload_id.clone())
            .text("chunk_index", chunk_index.to_string())
            .text("total_chunks", total_chunks.to_string())
            .text("filename", filename.clone())
            .part("file", part);

        let res = client
            .post(&url_chunk)
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("chunk {chunk_index} send: {e}"))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            let snippet: String = body.chars().take(300).collect();
            return Err(format!("chunk {chunk_index} HTTP {status}: {snippet}"));
        }
    }

    // Ask server to reassemble all chunks into a final file
    let form = multipart::Form::new()
        .text("upload_id", upload_id.clone())
        .text("total_chunks", total_chunks.to_string())
        .text("filename", filename.clone());

    let res = reqwest::Client::builder()
        .timeout(Duration::from_secs(300)) // assembly can take a moment for big files
        .build()
        .map_err(|e| e.to_string())?
        .post(&url_assemble)
        .header("Accept", "application/json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("assemble send: {e}"))?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(300).collect();
        return Err(format!("assemble HTTP {status}: {snippet}"));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("assemble parse: {e}"))?;
    Ok(json)
}

/// POST a video file to the Laravel `upload-video` route as a streaming multipart upload.
/// Uses streaming so large video files (100MB+) are not loaded fully into memory.
/// `field_name` defaults to "file" if empty. Timeout is 30 minutes.
#[tauri::command]
pub async fn upload_video_file(
    file_path: String,
    url: String,
    field_name: String,
) -> Result<(), String> {
    use reqwest::multipart;
    use std::path::Path;
    use std::time::Duration;

    let path = Path::new(&file_path);
    let field = if field_name.trim().is_empty() {
        "file".to_string()
    } else {
        field_name.clone()
    };

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();

    let mime = mime_for_video_path(path);

    let file_meta = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("stat file: {e}"))?;
    let file_size = file_meta.len();
    let file_size_mb = file_size as f64 / 1_048_576.0;

    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| format!("open file: {e}"))?;
    let stream = tokio_util::io::ReaderStream::new(file);

    let part = multipart::Part::stream_with_length(reqwest::Body::wrap_stream(stream), file_size)
        .file_name(filename.clone())
        .mime_str(mime)
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new().part(field.clone(), part);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1800)) // 30 min for large videos
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .post(&url)
        .header("Accept", "application/json")
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(400).collect();
        return Err(format!(
            "HTTP {status} (field: {field}, size: {file_size_mb:.1}MB): {snippet}"
        ));
    }

    Ok(())
}

fn mime_for_video_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("mts") | Some("m2ts") => "video/MP2T",
        _ => "application/octet-stream",
    }
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
