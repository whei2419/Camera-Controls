// Camera abstraction using libgphoto2 (open-source, no SDK download required)
// Install: brew install libgphoto2

use gphoto2::{Camera, Context};
use serde::{Deserialize, Serialize};

// ── Error type ────────────────────────────────────────────────────────────────
#[derive(Debug, thiserror::Error, Serialize)]
pub enum CameraError {
    #[error("{0}")]
    Gphoto(String),
    #[error("No camera connected")]
    NoCamera,
    #[error("Live view not active")]
    LiveViewOff,
}

impl From<gphoto2::Error> for CameraError {
    fn from(e: gphoto2::Error) -> Self {
        CameraError::Gphoto(e.to_string())
    }
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
// Stored in Tauri as tokio::sync::Mutex<CameraState> so async commands can
// hold the lock across .await points without deadlocking.
pub struct CameraState {
    pub context: Context,
    pub camera: Option<Camera>,
    pub live_view: bool,
}

// SAFETY: libgphoto2 is not thread-safe by itself, but we serialise
// every call through the tokio::sync::Mutex in AppState.
unsafe impl Send for CameraState {}
unsafe impl Sync for CameraState {}

impl CameraState {
    pub fn new() -> Result<Self, CameraError> {
        let context = Context::new()?;
        Ok(Self {
            context,
            camera: None,
            live_view: false,
        })
    }
}

// ── Camera helpers ────────────────────────────────────────────────────────────

/// List all cameras currently visible to libgphoto2.
pub async fn list_cameras(state: &CameraState) -> Result<Vec<CameraInfo>, CameraError> {
    let list = state.context.list_cameras().await?;
    let cameras = list
        .into_iter()
        .enumerate()
        .map(|(i, desc)| CameraInfo {
            index: i,
            name: desc.model().to_string(),
            port: desc.port().to_string(),
        })
        .collect();
    Ok(cameras)
}

/// Connect to the camera at `index` in the detected list.
pub async fn connect(state: &mut CameraState, index: usize) -> Result<CameraInfo, CameraError> {
    let list = state.context.list_cameras().await?;
    let descs: Vec<_> = list.into_iter().collect();
    let desc = descs.get(index).ok_or(CameraError::NoCamera)?;
    let camera = state.context.get_camera(desc.model(), desc.port()).await?;
    let info = CameraInfo {
        index,
        name: desc.model().to_string(),
        port: desc.port().to_string(),
    };
    state.camera = Some(camera);
    state.live_view = false;
    Ok(info)
}

/// Drop the active camera session.
pub fn disconnect(state: &mut CameraState) {
    state.camera = None;
    state.live_view = false;
}

// ── Config helpers ────────────────────────────────────────────────────────────

/// Get the string value of a named config widget.
async fn get_config_str(camera: &Camera, key: &str) -> String {
    let Ok(config) = camera.config().await else {
        return String::new();
    };
    let Ok(widget) = config.get_child_by_name(key) else {
        return String::new();
    };
    widget_value_to_string(&widget)
}

/// Get the list of choices for a named radio/menu config widget.
async fn get_config_choices(camera: &Camera, key: &str) -> Vec<String> {
    let Ok(config) = camera.config().await else {
        return vec![];
    };
    let Ok(widget) = config.get_child_by_name(key) else {
        return vec![];
    };
    widget_choices(&widget)
}

/// Set a named config widget to the given string value and apply it.
pub async fn set_config(state: &CameraState, key: &str, value: &str) -> Result<(), CameraError> {
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    let config = camera.config().await?;
    let widget = config
        .get_child_by_name(key)
        .map_err(|e| CameraError::Gphoto(e.to_string()))?;
    string_to_widget_value(&widget, value).map_err(|e| CameraError::Gphoto(e.to_string()))?;
    camera.set_config(&widget).await?;
    Ok(())
}

// ── Widget value conversion ───────────────────────────────────────────────────

fn widget_value_to_string(widget: &gphoto2::widget::Widget) -> String {
    use gphoto2::widget::WidgetValue;
    match widget.value() {
        Ok(WidgetValue::Text(s)) => s,
        Ok(WidgetValue::Radio(s)) => s,
        Ok(WidgetValue::Menu(s)) => s,
        Ok(WidgetValue::Range(f)) => format!("{f}"),
        Ok(WidgetValue::Toggle(b)) => {
            if b {
                "On".into()
            } else {
                "Off".into()
            }
        }
        Ok(WidgetValue::Date(n)) => format!("{n}"),
        _ => String::new(),
    }
}

fn widget_choices(widget: &gphoto2::widget::Widget) -> Vec<String> {
    widget
        .choices()
        .map(|choices| choices.map(|(c, _)| c.to_string()).collect())
        .unwrap_or_default()
}

fn string_to_widget_value(
    widget: &gphoto2::widget::Widget,
    value: &str,
) -> Result<(), gphoto2::Error> {
    use gphoto2::widget::WidgetType;
    use gphoto2::widget::WidgetValue;
    match widget.widget_type()? {
        WidgetType::Radio | WidgetType::Menu => {
            widget.set_value(WidgetValue::Radio(value.to_string()))
        }
        WidgetType::Text => widget.set_value(WidgetValue::Text(value.to_string())),
        WidgetType::Range => {
            if let Ok(f) = value.parse::<f32>() {
                widget.set_value(WidgetValue::Range(f))
            } else {
                Ok(())
            }
        }
        WidgetType::Toggle => {
            let b = matches!(value, "1" | "true" | "On" | "on");
            widget.set_value(WidgetValue::Toggle(b))
        }
        _ => Ok(()),
    }
}

// ── Shooting settings ─────────────────────────────────────────────────────────

pub async fn get_settings(state: &CameraState) -> Result<ShootingSettings, CameraError> {
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    Ok(ShootingSettings {
        iso: get_config_str(camera, "iso").await,
        aperture: get_config_str(camera, "aperture").await,
        shutter_speed: get_config_str(camera, "shutterspeed").await,
        white_balance: get_config_str(camera, "whitebalance").await,
        battery: get_config_str(camera, "batterylevel").await,
    })
}

pub async fn get_setting_options(state: &CameraState) -> Result<SettingOptions, CameraError> {
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    Ok(SettingOptions {
        iso: get_config_choices(camera, "iso").await,
        aperture: get_config_choices(camera, "aperture").await,
        shutter_speed: get_config_choices(camera, "shutterspeed").await,
        white_balance: get_config_choices(camera, "whitebalance").await,
    })
}

// ── Shutter ───────────────────────────────────────────────────────────────────

pub async fn take_picture(state: &CameraState) -> Result<(), CameraError> {
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    camera.capture_image().await?;
    Ok(())
}

// ── Live view ─────────────────────────────────────────────────────────────────

pub async fn start_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    // Enable viewfinder / EVF output to PC via config
    let _ = set_config_on_camera(camera, "viewfinder", "1").await;
    // Some Canon bodies use "eosremoterelease" or "evfmode"
    // gphoto2 handles this automatically on capture_preview
    state.live_view = true;
    Ok(())
}

pub async fn stop_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    if let Some(camera) = state.camera.as_ref() {
        let _ = set_config_on_camera(camera, "viewfinder", "0").await;
    }
    state.live_view = false;
    Ok(())
}

/// Returns a single live-view frame as a base64-encoded JPEG string.
pub async fn capture_live_view_frame(state: &CameraState) -> Result<String, CameraError> {
    if !state.live_view {
        return Err(CameraError::LiveViewOff);
    }
    let camera = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    let file = camera.capture_preview().await?;
    let data = file.get_data(&state.context).await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&data))
}

async fn set_config_on_camera(camera: &Camera, key: &str, value: &str) -> Result<(), CameraError> {
    let config = camera.config().await?;
    if let Ok(widget) = config.get_child_by_name(key) {
        let _ = string_to_widget_value(&widget, value);
        let _ = camera.set_config(&widget).await;
    }
    Ok(())
}
