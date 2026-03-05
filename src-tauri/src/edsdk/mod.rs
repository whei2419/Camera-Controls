// Safe Rust wrapper around the raw EDSDK FFI
// Provides an ergonomic interface for camera operations

pub mod ffi;
pub mod properties;

use std::ffi::CStr;
use std::sync::{Arc, Mutex};

use ffi::*;
use serde::{Deserialize, Serialize};

// ── Error type ────────────────────────────────────────────────────────────────
#[derive(Debug, thiserror::Error, Serialize)]
pub enum CameraError {
    #[error("EDSDK error: 0x{0:08X}")]
    Sdk(u32),
    #[error("No camera connected")]
    NoCamera,
    #[error("Session not open")]
    NoSession,
    #[error("SDK not initialised")]
    NotInitialised,
    #[error("Live view not active")]
    LiveViewOff,
    #[error("{0}")]
    Other(String),
}

impl From<EdsError> for CameraError {
    fn from(e: EdsError) -> Self {
        CameraError::Sdk(e)
    }
}

fn check(err: EdsError) -> Result<(), CameraError> {
    if err == EDS_ERR_OK {
        Ok(())
    } else {
        Err(CameraError::Sdk(err))
    }
}

// ── Camera info ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub index: usize,
    pub name: String,
    pub port: String,
}

// ── Shooting settings ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShootingSettings {
    pub av: u32,
    pub av_label: String,
    pub tv: u32,
    pub tv_label: String,
    pub iso: u32,
    pub iso_label: String,
    pub white_balance: i32,
    pub white_balance_label: String,
    pub battery: u32,
}

// ── SDK singleton ─────────────────────────────────────────────────────────────
pub struct Camera {
    camera_ref: EdsCameraRef,
    live_view_active: bool,
}

// SAFETY: We serialise all EDSDK calls through a Mutex<Option<Camera>>.
unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            let _ = EdsCloseSession(self.camera_ref);
            let _ = EdsRelease(self.camera_ref);
        }
    }
}

// ── Global state shared across Tauri commands ─────────────────────────────────
pub struct CameraState {
    pub sdk_initialised: bool,
    pub camera: Option<Camera>,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            sdk_initialised: false,
            camera: None,
        }
    }
}

pub type SharedCameraState = Arc<Mutex<CameraState>>;

// ── SDK lifecycle ─────────────────────────────────────────────────────────────
pub fn init_sdk(state: &mut CameraState) -> Result<(), CameraError> {
    if state.sdk_initialised {
        return Ok(());
    }
    unsafe { check(EdsInitializeSDK())? };
    state.sdk_initialised = true;
    Ok(())
}

pub fn terminate_sdk(state: &mut CameraState) -> Result<(), CameraError> {
    state.camera = None; // triggers Drop → CloseSession + Release
    if state.sdk_initialised {
        unsafe { check(EdsTerminateSDK())? };
        state.sdk_initialised = false;
    }
    Ok(())
}

// ── List available cameras ─────────────────────────────────────────────────────
pub fn list_cameras(state: &CameraState) -> Result<Vec<CameraInfo>, CameraError> {
    if !state.sdk_initialised {
        return Err(CameraError::NotInitialised);
    }

    unsafe {
        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        check(EdsGetCameraList(&mut camera_list))?;

        let mut count: EdsUInt32 = 0;
        check(EdsGetChildCount(camera_list, &mut count))?;

        let mut cameras = Vec::new();
        for i in 0..count {
            let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
            if EdsGetChildAtIndex(camera_list, i as EdsInt32, &mut cam_ref) != EDS_ERR_OK {
                continue;
            }
            let mut info = EdsDeviceInfo {
                szPortName: [0; 256],
                szDeviceDescription: [0; 256],
                deviceSubType: 0,
                reserved: 0,
            };
            if EdsGetDeviceInfo(cam_ref, &mut info) == EDS_ERR_OK {
                let name = CStr::from_ptr(info.szDeviceDescription.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                let port = CStr::from_ptr(info.szPortName.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                cameras.push(CameraInfo {
                    index: i as usize,
                    name,
                    port,
                });
            }
            EdsRelease(cam_ref);
        }
        EdsRelease(camera_list);
        Ok(cameras)
    }
}

// ── Connect / disconnect ──────────────────────────────────────────────────────
pub fn connect_camera(state: &mut CameraState, index: usize) -> Result<CameraInfo, CameraError> {
    if !state.sdk_initialised {
        init_sdk(state)?;
    }
    // Close any existing session
    state.camera = None;

    unsafe {
        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        check(EdsGetCameraList(&mut camera_list))?;

        let mut count: EdsUInt32 = 0;
        check(EdsGetChildCount(camera_list, &mut count))?;

        if index as EdsUInt32 >= count {
            EdsRelease(camera_list);
            return Err(CameraError::NoCamera);
        }

        let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
        check(EdsGetChildAtIndex(
            camera_list,
            index as EdsInt32,
            &mut cam_ref,
        ))?;
        EdsRelease(camera_list);

        let mut info = EdsDeviceInfo {
            szPortName: [0; 256],
            szDeviceDescription: [0; 256],
            deviceSubType: 0,
            reserved: 0,
        };
        EdsGetDeviceInfo(cam_ref, &mut info);

        check(EdsOpenSession(cam_ref))?;

        // Save to both camera + host by default
        let save_to: EdsUInt32 = kEdsSaveTo_Camera;
        check(EdsSetPropertyData(
            cam_ref,
            kEdsPropID_SaveTo,
            0,
            std::mem::size_of::<EdsUInt32>() as EdsUInt32,
            &save_to as *const _ as *const EdsVoid,
        ))?;

        let name = CStr::from_ptr(info.szDeviceDescription.as_ptr())
            .to_string_lossy()
            .into_owned();
        let port = CStr::from_ptr(info.szPortName.as_ptr())
            .to_string_lossy()
            .into_owned();

        let cam_info = CameraInfo {
            index,
            name,
            port: port.clone(),
        };

        state.camera = Some(Camera {
            camera_ref: cam_ref,
            live_view_active: false,
        });

        Ok(cam_info)
    }
}

pub fn disconnect_camera(state: &mut CameraState) -> Result<(), CameraError> {
    state.camera = None;
    Ok(())
}

// ── Get property helper ───────────────────────────────────────────────────────
unsafe fn get_property_u32(cam_ref: EdsCameraRef, prop_id: EdsUInt32) -> Result<u32, CameraError> {
    let mut value: EdsUInt32 = 0;
    check(EdsGetPropertyData(
        cam_ref,
        prop_id,
        0,
        std::mem::size_of::<EdsUInt32>() as EdsUInt32,
        &mut value as *mut _ as *mut EdsVoid,
    ))?;
    Ok(value)
}

unsafe fn set_property_u32(
    cam_ref: EdsCameraRef,
    prop_id: EdsUInt32,
    value: EdsUInt32,
) -> Result<(), CameraError> {
    check(EdsSetPropertyData(
        cam_ref,
        prop_id,
        0,
        std::mem::size_of::<EdsUInt32>() as EdsUInt32,
        &value as *const _ as *const EdsVoid,
    ))?;
    Ok(())
}

// ── Read current shooting settings ───────────────────────────────────────────
pub fn get_settings(state: &CameraState) -> Result<ShootingSettings, CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe {
        let av = get_property_u32(cam.camera_ref, kEdsPropID_Av).unwrap_or(0);
        let tv = get_property_u32(cam.camera_ref, kEdsPropID_Tv).unwrap_or(0);
        let iso = get_property_u32(cam.camera_ref, kEdsPropID_ISOSpeed).unwrap_or(0);
        let wb_raw = get_property_u32(cam.camera_ref, kEdsPropID_WhiteBalance).unwrap_or(0);
        let wb = wb_raw as i32;
        let battery = get_property_u32(cam.camera_ref, kEdsPropID_BatteryLevel).unwrap_or(0);

        Ok(ShootingSettings {
            av,
            av_label: properties::av_label(av),
            tv,
            tv_label: properties::tv_label(tv),
            iso,
            iso_label: properties::iso_label(iso),
            white_balance: wb,
            white_balance_label: properties::wb_label(wb),
            battery,
        })
    }
}

// ── Set individual settings ───────────────────────────────────────────────────
pub fn set_av(state: &mut CameraState, value: u32) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe { set_property_u32(cam.camera_ref, kEdsPropID_Av, value) }
}

pub fn set_tv(state: &mut CameraState, value: u32) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe { set_property_u32(cam.camera_ref, kEdsPropID_Tv, value) }
}

pub fn set_iso(state: &mut CameraState, value: u32) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe { set_property_u32(cam.camera_ref, kEdsPropID_ISOSpeed, value) }
}

pub fn set_white_balance(state: &mut CameraState, value: i32) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe { set_property_u32(cam.camera_ref, kEdsPropID_WhiteBalance, value as u32) }
}

// ── Shutter control ───────────────────────────────────────────────────────────
pub fn take_picture(state: &CameraState) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe {
        check(EdsSendCommand(
            cam.camera_ref,
            kEdsCameraCommand_TakePicture,
            0,
        ))
    }
}

pub fn press_shutter_halfway(state: &CameraState) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe {
        check(EdsSendCommand(
            cam.camera_ref,
            kEdsCameraCommand_ShutterButton_Halfway,
            0,
        ))
    }
}

pub fn release_shutter(state: &CameraState) -> Result<(), CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    unsafe {
        check(EdsSendCommand(
            cam.camera_ref,
            kEdsCameraCommand_ShutterButton_OFF,
            0,
        ))
    }
}

// ── Live view ─────────────────────────────────────────────────────────────────
pub fn start_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    let cam = state.camera.as_mut().ok_or(CameraError::NoCamera)?;
    unsafe {
        // Enable EVF mode on camera
        set_property_u32(cam.camera_ref, kEdsPropID_Evf_Mode, 1)?;
        // Route output to PC
        set_property_u32(cam.camera_ref, kEdsPropID_Evf_OutputDevice, kEdsEvfOutputDevice_PC)?;
    }
    cam.live_view_active = true;
    Ok(())
}

pub fn stop_live_view(state: &mut CameraState) -> Result<(), CameraError> {
    let cam = state.camera.as_mut().ok_or(CameraError::NoCamera)?;
    unsafe {
        set_property_u32(cam.camera_ref, kEdsPropID_Evf_OutputDevice, 0)?;
        set_property_u32(cam.camera_ref, kEdsPropID_Evf_Mode, 0)?;
    }
    cam.live_view_active = false;
    Ok(())
}

/// Returns a JPEG frame as base64-encoded string, ready for an <img src="data:image/jpeg;base64,...">
pub fn capture_live_view_frame(state: &CameraState) -> Result<String, CameraError> {
    let cam = state.camera.as_ref().ok_or(CameraError::NoCamera)?;
    if !cam.live_view_active {
        return Err(CameraError::LiveViewOff);
    }

    unsafe {
        let mut stream: EdsStreamRef = std::ptr::null_mut();
        check(EdsCreateMemoryStream(0, &mut stream))?;

        let mut evf_image: EdsEvfImageRef = std::ptr::null_mut();
        let create_result = EdsCreateEvfImageRef(stream, &mut evf_image);
        if create_result != EDS_ERR_OK {
            EdsRelease(stream);
            return Err(CameraError::Sdk(create_result));
        }

        let download_result = EdsDownloadEvfImage(cam.camera_ref, evf_image);
        EdsRelease(evf_image);

        if download_result != EDS_ERR_OK {
            EdsRelease(stream);
            return Err(CameraError::Sdk(download_result));
        }

        let mut ptr: *mut EdsVoid = std::ptr::null_mut();
        let mut length: EdsUInt64 = 0;

        EdsGetPointer(stream, &mut ptr);
        EdsGetLength(stream, &mut length);

        let slice = std::slice::from_raw_parts(ptr as *const u8, length as usize);
        let encoded = base64_encode(slice);

        EdsRelease(stream);
        Ok(encoded)
    }
}

// Simple base64 encoder (no external dep needed for this use-case)
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        let _ = result.len(); // suppress unused write warning
    }
    result
}
