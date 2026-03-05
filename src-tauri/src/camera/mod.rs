// Camera abstraction using DigiCamControl (Windows, open-source, no SDK required)
// Install DigiCamControl from: https://digicamcontrol.com/
// Enable "WebServer" plugin in DigiCamControl → Plugins and ensure it runs on port 5513.

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DC_BASE: &str = "http://localhost:5513";

// ── Error type ────────────────────────────────────────────────────────────────
#[derive(Debug, thiserror::Error, Serialize)]
pub enum CameraError {
    #[error("{0}")]
    Http(String),
    #[error("DigiCamControl is not running. Start it and enable the WebServer plugin.")]
    NotRunning,
    #[error("No camera connected")]
    NoCamera,
    #[error("Live view not active")]
    LiveViewOff,
}

impl From<reqwest::Error> for CameraError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_connect() {
            CameraError::NotRunning
        } else {
            CameraError::Http(e.to_string())
        }
    }
}

// ── DigiCamControl session JSON shapes ────────────────────────────────────────
#[derive(Debug, Deserialize, Default)]
struct DcProp {
    #[serde(rename = "Value", default)]
    value: String,
    #[serde(rename = "Values", default)]
    values: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DcProperties {
    #[serde(rename = "Iso", default)]
    iso: DcProp,
    #[serde(rename = "ShutterSpeed", default)]
    shutter_speed: DcProp,
    #[serde(rename = "Aperture", default)]
    aperture: DcProp,
    #[serde(rename = "WhiteBalance", default)]
    white_balance: DcProp,
    #[serde(rename = "Battery", default)]
    battery: DcProp,
}

#[derive(Debug, Deserialize)]
struct DcCamera {
    #[serde(rename = "DisplayName", default)]
    display_name: String,
    #[serde(rename = "Port", default)]
    port: String,
    #[serde(rename = "Properties", default)]
    properties: Option<DcProperties>,
}

#[derive(Debug, Deserialize, Default)]
struct DcSession {
    #[serde(rename = "ConnectedCamera")]
    connected_camera: Option<DcCamera>,
    #[serde(rename = "Cameras", default)]
    cameras: Vec<DcCamera>,
}

// ── Data types shared with frontend ──────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub index: usize,
    pub name: String,
    pub port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShootingSettings {
    pub iso: String,
    pub aperture: String,
    pub shutter_speed: String,
    pub white_balance: String,
    pub battery: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingOptions {
    pub iso: Vec<String>,
    pub aperture: Vec<String>,
    pub shutter_speed: Vec<String>,
    pub white_balance: Vec<String>,
}

// ── App state ─────────────────────────────────────────────────────────────────
pub struct CameraState {
    pub client: Client,
    pub connected: bool,
    pub live_view: bool,
}

impl CameraState {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            connected: false,
            live_view: false,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn fetch_session(client: &Client) -> Result<DcSession, CameraError> {
    let url = format!("{DC_BASE}/session.json");
    let text = client.get(&url).send().await?.text().await?;
    serde_json::from_str::<DcSession>(&text)
        .map_err(|e| CameraError::Http(format!("Parse error: {e} — response: {text}")))
}

async fn send_cmd(client: &Client, cmd: &str) -> Result<(), CameraError> {
    client
        .get(format!("{DC_BASE}/"))
        .query(&[("CMD", cmd)])
        .send()
        .await?;
    Ok(())
}

// ── Camera discovery ──────────────────────────────────────────────────────────

pub async fn list_cameras(state: &CameraState) -> Result<Vec<CameraInfo>, CameraError> {
    let session = fetch_session(&state.client).await?;
    if session.cameras.is_empty() {
        // Fallback: if a camera is actively connected, include it
        if let Some(cam) = &session.connected_camera {
            return Ok(vec![CameraInfo {
                index: 0,
                name: cam.display_name.clone(),
                port: cam.port.clone(),
            }]);
        }
    }
    let cameras = session
        .cameras
        .into_iter()
        .enumerate()
        .map(|(i, cam)| CameraInfo {
            index: i,
            name: cam.display_name,
            port: cam.port,
        })
        .collect();
    Ok(cameras)
}

// ── Connection ────────────────────────────────────────────────────────────────

/// "Connect" just verifies DigiCamControl has an active camera at that index
/// and marks our state as connected.
pub async fn connect(state: &mut CameraState, index: usize) -> Result<CameraInfo, CameraError> {
    let cameras = list_cameras(state).await?;
    let info = cameras
        .into_iter()
        .find(|c| c.index == index)
        .ok_or(CameraError::NoCamera)?;
    state.connected = true;
    Ok(info)
}

pub fn disconnect(state: &mut CameraState) {
    state.connected = false;
    state.live_view = false;
}

// ── Settings ──────────────────────────────────────────────────────────────────

pub async fn get_settings(state: &CameraState) -> Result<ShootingSettings, CameraError> {
    if !state.connected {
        return Err(CameraError::NoCamera);
    }
    let session = fetch_session(&state.client).await?;
    let cam = session.connected_camera.ok_or(CameraError::NoCamera)?;
    let props = cam.properties.unwrap_or_default();
    Ok(ShootingSettings {
        iso: props.iso.value,
        aperture: props.aperture.value,
        shutter_speed: props.shutter_speed.value,
        white_balance: props.white_balance.value,
        battery: props.battery.value,
    })
}

pub async fn get_setting_options(state: &CameraState) -> Result<SettingOptions, CameraError> {
    if !state.connected {
        return Err(CameraError::NoCamera);
    }
    let session = fetch_session(&state.client).await?;
    let cam = session.connected_camera.ok_or(CameraError::NoCamera)?;
    let props = cam.properties.unwrap_or_default();
    Ok(SettingOptions {
        iso: props.iso.values,
        aperture: props.aperture.values,
        shutter_speed: props.shutter_speed.values,
        white_balance: props.white_balance.values,
    })
}

// ── Set individual settings ───────────────────────────────────────────────────

pub async fn set_config(state: &CameraState, prop: &str, value: &str) -> Result<(), CameraError> {
    if !state.connected {
        return Err(CameraError::NoCamera);
    }
    state
        .client
        .get(format!("{DC_BASE}/"))
        .query(&[("CMD", "SetProperty"), ("property", prop), ("value", value)])
        .send()
        .await?;
    Ok(())
}

// ── Shutter ───────────────────────────────────────────────────────────────────

pub async fn take_picture(state: &CameraState) -> Result<(), CameraError> {
    if !state.connected {
        return Err(CameraError::NoCamera);
    }
    send_cmd(&state.client, "Capture").await
}

// ── Live view ─────────────────────────────────────────────────────────────────

pub async fn start_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    if !state.connected {
        return Err(CameraError::NoCamera);
    }
    send_cmd(&state.client, "LiveViewWnd_Show").await?;
    state.live_view = true;
    Ok(())
}

pub async fn stop_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    if state.live_view {
        let _ = send_cmd(&state.client, "LiveViewWnd_Hide").await;
    }
    state.live_view = false;
    Ok(())
}

/// Fetches `/liveview.jpg` from DigiCamControl and returns it as a
/// base64-encoded JPEG string ready for `<img src="data:image/jpeg;base64,...">`.
pub async fn capture_live_view_frame(state: &CameraState) -> Result<String, CameraError> {
    if !state.live_view {
        return Err(CameraError::LiveViewOff);
    }
    let bytes = state
        .client
        .get(format!("{DC_BASE}/liveview.jpg"))
        .send()
        .await?
        .bytes()
        .await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}
