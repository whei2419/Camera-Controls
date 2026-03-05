// Raw FFI bindings for Canon EDSDK (C API)
// Canon EDSDK must be obtained from Canon's developer portal:
// https://developercommunity.usa.canon.com/
// Place EDSDK.framework (macOS) or EDSDK.dll (Windows) in src-tauri/libs/

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_imports
)]

use std::os::raw::{c_char, c_void};

// ── Primitive types ──────────────────────────────────────────────────────────
pub type EdsUInt32 = u32;
pub type EdsInt32 = i32;
pub type EdsUInt64 = u64;
pub type EdsBool = u32;
pub type EdsError = EdsUInt32;
pub type EdsVoid = c_void;
pub type EdsChar = c_char;

// ── Reference handles ────────────────────────────────────────────────────────
pub type EdsBaseRef = *mut c_void;
pub type EdsCameraListRef = EdsBaseRef;
pub type EdsCameraRef = EdsBaseRef;
pub type EdsStreamRef = EdsBaseRef;
pub type EdsEvfImageRef = EdsBaseRef;

// ── Error codes ───────────────────────────────────────────────────────────────
pub const EDS_ERR_OK: EdsError = 0x00000000;
pub const EDS_ERR_UNIMPLEMENTED: EdsError = 0x00000001;
pub const EDS_ERR_DEVICE_BUSY: EdsError = 0x00000882;
pub const EDS_ERR_OBJECT_NOTREADY: EdsError = 0x00000881;
pub const EDS_ERR_COMM_DISCONNECTED: EdsError = 0x00002007;

// ── Property IDs ─────────────────────────────────────────────────────────────
pub const kEdsPropID_ProductName: EdsUInt32 = 0x00000002;
pub const kEdsPropID_BodyIDEx: EdsUInt32 = 0x00000015;
pub const kEdsPropID_Av: EdsUInt32 = 0x00000401;
pub const kEdsPropID_Tv: EdsUInt32 = 0x00000402;
pub const kEdsPropID_ISOSpeed: EdsUInt32 = 0x00000403;
pub const kEdsPropID_MeteringMode: EdsUInt32 = 0x00000404;
pub const kEdsPropID_ExposureCompensation: EdsUInt32 = 0x00000407;
pub const kEdsPropID_WhiteBalance: EdsUInt32 = 0x00000410;
pub const kEdsPropID_ColorTemperature: EdsUInt32 = 0x00000411;
pub const kEdsPropID_ImageQuality: EdsUInt32 = 0x00000100;
pub const kEdsPropID_AEModeSelect: EdsUInt32 = 0x00000435;
pub const kEdsPropID_SaveTo: EdsUInt32 = 0x0000000b;
pub const kEdsPropID_BatteryLevel: EdsUInt32 = 0x00000012;
pub const kEdsPropID_Evf_OutputDevice: EdsUInt32 = 0x00000500;
pub const kEdsPropID_Evf_Mode: EdsUInt32 = 0x00000501;
pub const kEdsPropID_Evf_AFMode: EdsUInt32 = 0x0000050e;

// ── Camera commands ───────────────────────────────────────────────────────────
pub const kEdsCameraCommand_TakePicture: EdsUInt32 = 0x00000000;
pub const kEdsCameraCommand_ShutterButton_Completely: EdsUInt32 = 0x00000003;
pub const kEdsCameraCommand_ShutterButton_Halfway: EdsUInt32 = 0x00000001;
pub const kEdsCameraCommand_ShutterButton_OFF: EdsUInt32 = 0x00000000;
pub const kEdsCameraCommand_DoEvfAf: EdsUInt32 = 0x00000102;

// ── Save destinations ─────────────────────────────────────────────────────────
pub const kEdsSaveTo_Camera: EdsUInt32 = 1;
pub const kEdsSaveTo_Host: EdsUInt32 = 2;
pub const kEdsSaveTo_Both: EdsUInt32 = 3;

// ── Live View output device ───────────────────────────────────────────────────
pub const kEdsEvfOutputDevice_TFT: EdsUInt32 = 0x00000001;
pub const kEdsEvfOutputDevice_PC: EdsUInt32 = 0x00000002;

// ── Events ────────────────────────────────────────────────────────────────────
pub const kEdsObjectEvent_All: EdsUInt32 = 0x00000200;
pub const kEdsPropertyEvent_All: EdsUInt32 = 0x00000100;
pub const kEdsStateEvent_All: EdsUInt32 = 0x00000300;
pub const kEdsStateEvent_WillSoonShutDown: EdsUInt32 = 0x00000302;

// ── Device info struct ────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Debug, Clone)]
pub struct EdsDeviceInfo {
    pub szPortName: [c_char; 256],
    pub szDeviceDescription: [c_char; 256],
    pub deviceSubType: EdsUInt32,
    pub reserved: EdsUInt32,
}

// ── Callback types ────────────────────────────────────────────────────────────
pub type EdsObjectEventHandler =
    extern "C" fn(inEvent: EdsUInt32, inRef: EdsBaseRef, inContext: *mut c_void) -> EdsError;
pub type EdsPropertyEventHandler = extern "C" fn(
    inEvent: EdsUInt32,
    inPropertyID: EdsUInt32,
    inParam: EdsUInt32,
    inContext: *mut c_void,
) -> EdsError;
pub type EdsStateEventHandler =
    extern "C" fn(inEvent: EdsUInt32, inEventData: EdsUInt32, inContext: *mut c_void) -> EdsError;

// ── EDSDK function declarations ───────────────────────────────────────────────
#[link(name = "EDSDK")]
extern "C" {
    pub fn EdsInitializeSDK() -> EdsError;
    pub fn EdsTerminateSDK() -> EdsError;

    pub fn EdsGetCameraList(outCameraList: *mut EdsCameraListRef) -> EdsError;
    pub fn EdsGetChildCount(inRef: EdsBaseRef, outCount: *mut EdsUInt32) -> EdsError;
    pub fn EdsGetChildAtIndex(
        inRef: EdsBaseRef,
        inIndex: EdsInt32,
        outRef: *mut EdsBaseRef,
    ) -> EdsError;
    pub fn EdsRelease(inRef: EdsBaseRef) -> EdsError;

    pub fn EdsGetDeviceInfo(
        inCameraRef: EdsCameraRef,
        outDeviceInfo: *mut EdsDeviceInfo,
    ) -> EdsError;
    pub fn EdsOpenSession(inCameraRef: EdsCameraRef) -> EdsError;
    pub fn EdsCloseSession(inCameraRef: EdsCameraRef) -> EdsError;

    pub fn EdsGetPropertySize(
        inRef: EdsBaseRef,
        inPropertyID: EdsUInt32,
        inParam: EdsInt32,
        outDataType: *mut EdsUInt32,
        outSize: *mut EdsUInt32,
    ) -> EdsError;
    pub fn EdsGetPropertyData(
        inRef: EdsBaseRef,
        inPropertyID: EdsUInt32,
        inParam: EdsInt32,
        inPropertySize: EdsUInt32,
        outPropertyData: *mut EdsVoid,
    ) -> EdsError;
    pub fn EdsSetPropertyData(
        inRef: EdsBaseRef,
        inPropertyID: EdsUInt32,
        inParam: EdsInt32,
        inPropertySize: EdsUInt32,
        inPropertyData: *const EdsVoid,
    ) -> EdsError;

    pub fn EdsSendCommand(
        inCameraRef: EdsCameraRef,
        inCommand: EdsUInt32,
        inParam: EdsInt32,
    ) -> EdsError;

    pub fn EdsSetObjectEventHandler(
        inCameraRef: EdsCameraRef,
        inEvent: EdsUInt32,
        inObjectEventCallback: Option<EdsObjectEventHandler>,
        inContext: *mut c_void,
    ) -> EdsError;
    pub fn EdsSetPropertyEventHandler(
        inCameraRef: EdsCameraRef,
        inEvent: EdsUInt32,
        inPropertyEventCallback: Option<EdsPropertyEventHandler>,
        inContext: *mut c_void,
    ) -> EdsError;
    pub fn EdsSetCameraStateEventHandler(
        inCameraRef: EdsCameraRef,
        inEvent: EdsUInt32,
        inStateEventCallback: Option<EdsStateEventHandler>,
        inContext: *mut c_void,
    ) -> EdsError;

    // Live view
    pub fn EdsCreateMemoryStream(inBufferSize: EdsUInt64, outStream: *mut EdsStreamRef)
        -> EdsError;
    pub fn EdsCreateEvfImageRef(
        inStreamRef: EdsStreamRef,
        outEvfImageRef: *mut EdsEvfImageRef,
    ) -> EdsError;
    pub fn EdsDownloadEvfImage(
        inCameraRef: EdsCameraRef,
        inEvfImageRef: EdsEvfImageRef,
    ) -> EdsError;
    pub fn EdsGetPointer(inStreamRef: EdsStreamRef, outPointer: *mut *mut EdsVoid) -> EdsError;
    pub fn EdsGetLength(inStreamRef: EdsStreamRef, outLength: *mut EdsUInt64) -> EdsError;
}
