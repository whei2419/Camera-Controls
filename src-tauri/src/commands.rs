// Tauri commands – exposed to the Vue frontend via invoke()

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
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

/// Read `length` bytes from `file_path` starting at `offset`, returned as a base64 string.
/// Used by the Vue frontend to read local file chunks for browser-side fetch() uploads.
#[tauri::command]
pub async fn read_file_chunk(
    file_path: String,
    offset: u64,
    length: u64,
) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

    let mut file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("open: {e}"))?;

    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("seek: {e}"))?;

    let mut buf = vec![0u8; length as usize];
    let mut total_read = 0usize;
    while total_read < length as usize {
        let n = file
            .read(&mut buf[total_read..])
            .await
            .map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            break;
        }
        total_read += n;
    }
    buf.truncate(total_read);

    Ok(BASE64_STANDARD.encode(&buf))
}

/// Read `length` bytes from `file_path` starting at `offset`, returned as raw binary.
/// The JavaScript caller receives an `ArrayBuffer` — no base64 encode/decode overhead.
#[tauri::command]
pub async fn read_file_chunk_bytes(
    file_path: String,
    offset: u64,
    length: u64,
) -> Result<tauri::ipc::Response, String> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

    let mut file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| format!("open: {e}"))?;

    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("seek: {e}"))?;

    let mut buf = vec![0u8; length as usize];
    let mut total_read = 0usize;
    while total_read < length as usize {
        let n = file
            .read(&mut buf[total_read..])
            .await
            .map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            break;
        }
        total_read += n;
    }
    buf.truncate(total_read);

    Ok(tauri::ipc::Response::new(buf))
}

// ---------------------------------------------------------------------------
// Native parallel chunked upload — sends raw bytes directly from Rust.
// Eliminates the base64 IPC round-trip that the browser-fetch approach needs.
// Metadata travels in URL query params (parsed by Apache before touching the
// body), body is application/octet-stream with a known Content-Length, which
// reqwest streams immediately — satisfying mod_reqtimeout's minrate check.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct UploadProgress {
    done: u64,
    total: u64,
}

/// Upload a video file in parallel binary chunks directly from Rust.
/// Emits "upload-progress" events so the frontend can show progress.
#[tauri::command]
pub async fn upload_video_chunked_native(
    app: tauri::AppHandle,
    file_path: String,
    file_size: u64,
    url_chunk: String,
    url_assemble: String,
) -> Result<serde_json::Value, String> {
    use std::path::Path;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    use tauri::Emitter;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    const CHUNK_SIZE: u64 = 8 * 1024 * 1024; // 8 MB
    const CONCURRENCY: usize = 5;

    let path = Path::new(&file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();

    let total_chunks = file_size.div_ceil(CHUNK_SIZE);

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let upload_id = format!("vid_{ts}");

    let client = Arc::new(
        reqwest::Client::builder()
            .use_native_tls()
            .timeout(std::time::Duration::from_secs(600))
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?,
    );

    let semaphore = Arc::new(Semaphore::new(CONCURRENCY));
    let done_count = Arc::new(AtomicU64::new(0));
    let mut join_set: JoinSet<Result<(), String>> = JoinSet::new();

    for chunk_index in 0..total_chunks {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;

        let client = client.clone();
        let file_path = file_path.clone();
        let url_chunk = url_chunk.clone();
        let upload_id = upload_id.clone();
        let filename = filename.clone();
        let done_count = done_count.clone();
        let app = app.clone();

        join_set.spawn(async move {
            let _permit = permit; // dropped when task completes, releasing the semaphore slot

            let offset = chunk_index * CHUNK_SIZE;
            let length = CHUNK_SIZE.min(file_size - offset);

            // Read chunk with fill-loop so we always get exactly `length` bytes
            let mut file = tokio::fs::File::open(&file_path)
                .await
                .map_err(|e| format!("open: {e}"))?;
            file.seek(SeekFrom::Start(offset))
                .await
                .map_err(|e| format!("seek: {e}"))?;

            let mut buf = vec![0u8; length as usize];
            let mut total_read = 0usize;
            while total_read < length as usize {
                let n = file
                    .read(&mut buf[total_read..])
                    .await
                    .map_err(|e| format!("read: {e}"))?;
                if n == 0 {
                    break;
                }
                total_read += n;
            }
            buf.truncate(total_read);

            let actual_len = buf.len() as u64;
            let params = [
                ("upload_id", upload_id.as_str()),
                ("chunk_index", &chunk_index.to_string()),
                ("total_chunks", &total_chunks.to_string()),
                ("filename", filename.as_str()),
            ];
            let url =
                reqwest::Url::parse_with_params(&url_chunk, &params).map_err(|e| e.to_string())?;

            let res = client
                .post(url)
                .header("Content-Type", "application/octet-stream")
                .header("Content-Length", actual_len.to_string())
                .header("Accept", "application/json")
                .body(buf)
                .send()
                .await
                .map_err(|e| format!("send chunk {chunk_index}: {e}"))?;

            if !res.status().is_success() {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                return Err(format!("chunk {chunk_index} failed: {status} — {body}"));
            }

            let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app.emit(
                "upload-progress",
                UploadProgress {
                    done,
                    total: total_chunks,
                },
            );

            Ok(())
        });
    }

    // Await all tasks; surface the first error
    while let Some(result) = join_set.join_next().await {
        result.map_err(|e| format!("task panic: {e}"))??;
    }

    // All chunks stored — trigger server-side assembly
    let assemble_body = serde_json::json!({
        "upload_id":    upload_id,
        "total_chunks": total_chunks,
        "filename":     filename,
    });

    let assemble_res = client
        .post(&url_assemble)
        .header("Accept", "application/json")
        .json(&assemble_body)
        .send()
        .await
        .map_err(|e| format!("assemble send: {e}"))?;

    if !assemble_res.status().is_success() {
        let status = assemble_res.status();
        let body = assemble_res.text().await.unwrap_or_default();
        return Err(format!("assemble failed: {status} — {body}"));
    }

    assemble_res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())
}

/// POST the capture file to the Laravel `upload-capture` route.
#[tauri::command]
pub async fn upload_capture_file(
    file_path: String,
    url: String,
    _field_name: String,
) -> Result<(), String> {
    use std::path::Path;
    use std::time::Duration;

    let path = Path::new(&file_path);
    let file_bytes = std::fs::read(path).map_err(|e| format!("read file: {e}"))?;
    let mime = mime_for_capture_path(path);

    let client = reqwest::Client::builder()
        .use_native_tls()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let b64 = BASE64_STANDARD.encode(&file_bytes);
    let data_uri = format!("data:{mime};base64,{b64}");

    let json_body = serde_json::json!({ "image": data_uri });

    let res = client
        .post(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json; charset=utf-8")
        .json(&json_body)
        .send()
        .await
        .map_err(|e| format!("send error: {e}"))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "HTTP {}: {}",
            res.status(),
            res.text().await.unwrap_or_default()
        ))
    }
}

/// Wait until a file's size stops changing.
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

    Err(format!("file did not stabilize within {}s", max_secs))
}

/// Upload a video file in sequential 1 MB base64-JSON chunks.
/// Chunks are sent as application/json (not multipart) to bypass PHP's $_FILES stack,
/// which fails on Apache/mod_reqtimeout with UPLOAD_ERR_PARTIAL.
#[tauri::command]
pub async fn upload_video_chunked(
    file_path: String,
    url_chunk: String,
    url_assemble: String,
) -> Result<serde_json::Value, String> {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

    const CHUNK_SIZE: u64 = 1024 * 1024; // 1 MB
    let path = Path::new(&file_path);

    let meta = tokio::fs::metadata(path).await.map_err(|e| e.to_string())?;
    let file_size = meta.len();
    let total_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let upload_id = format!("vid_{ts}");

    let client = reqwest::Client::builder()
        .use_native_tls()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| e.to_string())?;

    // Sequential Upload Loop — JSON + base64, bypasses PHP $_FILES entirely
    for chunk_index in 0..total_chunks {
        file.seek(SeekFrom::Start(chunk_index * CHUNK_SIZE))
            .await
            .map_err(|e| e.to_string())?;

        // Fill the buffer completely (read() may return fewer bytes than requested).
        let mut buf = vec![0u8; CHUNK_SIZE as usize];
        let mut total_read = 0usize;
        while total_read < CHUNK_SIZE as usize {
            let n = file
                .read(&mut buf[total_read..])
                .await
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break; // EOF
            }
            total_read += n;
        }
        buf.truncate(total_read);

        let chunk_b64 = BASE64_STANDARD.encode(&buf);

        let body = serde_json::json!({
            "upload_id":    upload_id,
            "chunk_index":  chunk_index,
            "total_chunks": total_chunks,
            "filename":     filename,
            "chunk_data":   chunk_b64,
        });

        let res = client
            .post(&url_chunk)
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            return Err(format!(
                "Failed chunk {chunk_index}: {status} — {body_text}"
            ));
        }

        eprintln!(
            "[upload_video_chunked] chunk {}/{} ok",
            chunk_index + 1,
            total_chunks
        );
    }

    // Assemble — plain JSON, no binary
    let assemble_body = serde_json::json!({
        "upload_id":    upload_id,
        "total_chunks": total_chunks,
        "filename":     filename,
    });

    let res = client
        .post(&url_assemble)
        .header("Accept", "application/json")
        .json(&assemble_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();
        return Err(format!("Assemble failed: {status} — {body_text}"));
    }

    res.json().await.map_err(|e| e.to_string())
}

/// Upload a video file using resumable chunks.
#[tauri::command]
pub async fn upload_video_resumable(
    file_path: String,
    url: String,
) -> Result<serde_json::Value, String> {
    use reqwest::multipart;
    use std::path::Path;
    use std::time::Duration;
    use tokio::io::AsyncReadExt;

    const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

    let path = Path::new(&file_path);
    let file_size = tokio::fs::metadata(&path)
        .await
        .map_err(|e| format!("metadata: {e}"))?
        .len() as usize;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("invalid filename")?;
    let filename_safe = filename
        .replace(' ', "_")
        .replace('(', "_")
        .replace(')', "_")
        .replace('[', "_")
        .replace(']', "_");

    let total_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let upload_id = format!("{}-{}", file_size, filename_safe.replace('.', ""));

    let client = reqwest::Client::builder()
        .use_native_tls()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| format!("open: {e}"))?;

    for chunk_number in 1..=total_chunks {
        // Use a fill-loop: a single read() may return fewer bytes than requested
        // even in the middle of a file, which would corrupt reassembly on the server.
        let mut buf = vec![0u8; CHUNK_SIZE];
        let mut total_read = 0usize;
        loop {
            let n = file
                .read(&mut buf[total_read..])
                .await
                .map_err(|e| format!("read chunk {chunk_number}: {e}"))?;
            if n == 0 {
                break; // EOF
            }
            total_read += n;
            if total_read >= CHUNK_SIZE {
                break;
            }
        }
        buf.truncate(total_read);
        let n = total_read;
        if n == 0 {
            break;
        }

        let part = multipart::Part::bytes(buf)
            .file_name(filename_safe.clone())
            .mime_str("application/octet-stream")
            .map_err(|e| format!("mime: {e}"))?;
        let form = multipart::Form::new()
            .text("resumableChunkNumber", chunk_number.to_string())
            .text("resumableChunkSize", CHUNK_SIZE.to_string())
            .text("resumableCurrentChunkSize", n.to_string())
            .text("resumableTotalSize", file_size.to_string())
            .text("resumableTotalChunks", total_chunks.to_string())
            .text("resumableIdentifier", upload_id.clone())
            .text("resumableFilename", filename_safe.clone())
            .text("resumableRelativePath", filename_safe.clone())
            .part("file", part);

        let res = client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("send: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("chunk {} HTTP {}", chunk_number, res.status()));
        }
    }

    Ok(serde_json::json!({"success": true, "chunks": total_chunks}))
}

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
        field_name
    };
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4")
        .to_string();
    let mime = mime_for_video_path(path);

    let file_bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let part = multipart::Part::bytes(file_bytes)
        .file_name(filename)
        .mime_str(mime)
        .map_err(|e| e.to_string())?;
    let form = multipart::Form::new().part(field, part);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1800))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()));
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
        _ => "application/octet-stream",
    }
}

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
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

#[tauri::command]
pub async fn print_file(path: String, printer: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("mspaint")
        .args(["/pt", &path, &printer])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
