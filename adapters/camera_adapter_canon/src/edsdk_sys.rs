#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_void};

pub type EdsError = u32;
pub type EdsUInt32 = u32;
pub type EdsInt32 = i32;
pub type EdsUInt64 = u64;
pub type EdsInt64 = i64;
pub type EdsBool = i32;
pub type EdsPropertyID = u32;

pub type EdsBaseRef = *mut c_void;
pub type EdsCameraListRef = EdsBaseRef;
pub type EdsCameraRef = EdsBaseRef;
pub type EdsStreamRef = EdsBaseRef;
pub type EdsEvfImageRef = EdsBaseRef;
pub type EdsImageRef = EdsBaseRef;

pub const EDS_ERR_OK: EdsError = 0x00000000;
pub const EDS_ERR_UNIMPLEMENTED: EdsError = 0x00000001;
pub const EDS_ERR_INTERNAL_ERROR: EdsError = 0x00000002;
pub const EDS_ERR_MEM_ALLOC_FAILED: EdsError = 0x00000003;
pub const EDS_ERR_MEM_FREE_FAILED: EdsError = 0x00000004;
pub const EDS_ERR_OPERATION_CANCELLED: EdsError = 0x00000005;
pub const EDS_ERR_INCOMPATIBLE_VERSION: EdsError = 0x00000006;
pub const EDS_ERR_NOT_SUPPORTED: EdsError = 0x00000007;
pub const EDS_ERR_UNEXPECTED_EXCEPTION: EdsError = 0x00000008;
pub const EDS_ERR_PROTECTION_VIOLATION: EdsError = 0x00000009;
pub const EDS_ERR_MISSING_SUBCOMPONENT: EdsError = 0x0000000A;
pub const EDS_ERR_SELECTION_UNAVAILABLE: EdsError = 0x0000000B;
pub const EDS_ERR_DEVICE_NOT_FOUND: EdsError = 0x00000080;
pub const EDS_ERR_DEVICE_BUSY: EdsError = 0x00000082;
pub const EDS_ERR_DEVICE_INVALID: EdsError = 0x00000081;
pub const EDS_ERR_SESSION_ALREADY_OPEN: EdsError = 0x00002003;
pub const EDS_ERR_SESSION_NOT_OPEN: EdsError = 0x00002004;
pub const EDS_ERR_OBJECT_NOTREADY: EdsError = 0x000000A4;
pub const EDS_ERR_COMM_PORT_IS_IN_USE: EdsError = 0x000000C0;
pub const EDS_ERR_COMM_DISCONNECTED: EdsError = 0x000000C1;
pub const EDS_ERR_COMM_DEVICE_INCOMPATIBLE: EdsError = 0x000000C2;
pub const EDS_ERR_COMM_BUFFER_FULL: EdsError = 0x000000C3;
pub const EDS_ERR_COMM_USB_BUS_ERR: EdsError = 0x000000C4;

// Property IDs
pub const kEdsPropID_Unknown: EdsPropertyID = 0x0000ffff;
pub const kEdsPropID_ProductName: EdsPropertyID = 0x00000002;
pub const kEdsPropID_BodyIDEx: EdsPropertyID = 0x00000015;
pub const kEdsPropID_BatteryLevel: EdsPropertyID = 0x00000004;
pub const kEdsPropID_BatteryQuality: EdsPropertyID = 0x00000010;
pub const kEdsPropID_SaveTo: EdsPropertyID = 0x0000000b;
pub const kEdsPropID_CurrentStorage: EdsPropertyID = 0x0000000c;
pub const kEdsPropID_CurrentFolder: EdsPropertyID = 0x0000000d;
pub const kEdsPropID_AEMode: EdsPropertyID = 0x00000100;
pub const kEdsPropID_AEModeSelect: EdsPropertyID = 0x00000138;
pub const kEdsPropID_DriveMode: EdsPropertyID = 0x00000102;
pub const kEdsPropID_ISOSpeed: EdsPropertyID = 0x00000103;
pub const kEdsPropID_MeteringMode: EdsPropertyID = 0x00000104;
pub const kEdsPropID_AFMode: EdsPropertyID = 0x00000105;
pub const kEdsPropID_Av: EdsPropertyID = 0x00000106;
pub const kEdsPropID_Tv: EdsPropertyID = 0x00000107;
pub const kEdsPropID_ExposureCompensation: EdsPropertyID = 0x00000108;
pub const kEdsPropID_WhiteBalance: EdsPropertyID = 0x00000109;
pub const kEdsPropID_ColorTemperature: EdsPropertyID = 0x0000010a;
pub const kEdsPropID_ImageQuality: EdsPropertyID = 0x00000101;
pub const kEdsPropID_Evf_OutputDevice: EdsPropertyID = 0x00000500;
pub const kEdsPropID_Evf_Mode: EdsPropertyID = 0x00000501;
pub const kEdsPropID_Evf_CoordinateSystem: EdsPropertyID = 0x00000540;
pub const kEdsPropID_Evf_Zoom: EdsPropertyID = 0x00000541;
pub const kEdsPropID_Evf_ZoomPosition: EdsPropertyID = 0x00000542;
pub const kEdsPropID_Evf_ZoomRect: EdsPropertyID = 0x00000545;
pub const kEdsPropID_Evf_ImagePosition: EdsPropertyID = 0x00000547;
pub const kEdsPropID_Evf_Histogram: EdsPropertyID = 0x0000050a;
pub const kEdsPropID_Evf_HistogramStatus: EdsPropertyID = 0x0000050b;

// EVF Zoom
pub const kEdsEvfZoom_Fit: EdsUInt32 = 1;
pub const kEdsEvfZoom_x5: EdsUInt32 = 5;
pub const kEdsEvfZoom_x10: EdsUInt32 = 10;

// EVF Output Device Flags
pub const kEdsEvfOutputDevice_TFT: EdsUInt32 = 1;
pub const kEdsEvfOutputDevice_PC: EdsUInt32 = 2;
pub const kEdsEvfOutputDevice_MOBILE: EdsUInt32 = 4;
pub const kEdsEvfOutputDevice_PC_Small: EdsUInt32 = 8;
pub const kEdsEvfOutputDevice_MOBILE_Small: EdsUInt32 = 16;

// EVF Mode
pub const kEdsEvfMode_Disable: EdsUInt32 = 0;
pub const kEdsEvfMode_Enable: EdsUInt32 = 1;

// Camera Commands
pub const kEdsCameraCommand_TakePicture: EdsUInt32 = 0x00000000;
pub const kEdsCameraCommand_ExtendShutDownTimer: EdsUInt32 = 0x00000001;
pub const kEdsCameraCommand_BulbStart: EdsUInt32 = 0x00000002;
pub const kEdsCameraCommand_BulbEnd: EdsUInt32 = 0x00000003;
pub const kEdsCameraCommand_DoEvfAf: EdsUInt32 = 0x00000102;
pub const kEdsCameraCommand_DriveLensEvf: EdsUInt32 = 0x00000103;
pub const kEdsCameraCommand_DoClickWBEvf: EdsUInt32 = 0x00000104;
pub const kEdsCameraCommand_PressShutterButton: EdsUInt32 = 0x00000004;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EdsDeviceInfo {
    pub szPortName: [c_char; 256],
    pub szDeviceDescription: [c_char; 256],
    pub DeviceSubType: EdsUInt32,
    pub reserved: EdsUInt32,
}

impl Default for EdsDeviceInfo {
    fn default() -> Self {
        Self {
            szPortName: [0; 256],
            szDeviceDescription: [0; 256],
            DeviceSubType: 0,
            reserved: 0,
        }
    }
}

#[cfg_attr(target_os = "windows", link(name = "EDSDK"))]
unsafe extern "system" {
    pub fn EdsInitializeSDK() -> EdsError;
    pub fn EdsTerminateSDK() -> EdsError;

    pub fn EdsGetCameraList(outCameraList: *mut EdsCameraListRef) -> EdsError;
    pub fn EdsGetChildCount(inRef: EdsBaseRef, outCount: *mut EdsUInt32) -> EdsError;
    pub fn EdsGetChildAtIndex(
        inRef: EdsBaseRef,
        inIndex: EdsInt32,
        outRef: *mut EdsBaseRef,
    ) -> EdsError;

    pub fn EdsGetDeviceInfo(
        inCameraRef: EdsCameraRef,
        outDeviceInfo: *mut EdsDeviceInfo,
    ) -> EdsError;

    pub fn EdsOpenSession(inCameraRef: EdsCameraRef) -> EdsError;
    pub fn EdsCloseSession(inCameraRef: EdsCameraRef) -> EdsError;

    pub fn EdsRelease(inRef: EdsBaseRef) -> EdsUInt32;
    pub fn EdsRetain(inRef: EdsBaseRef) -> EdsUInt32;

    pub fn EdsGetPropertyData(
        inRef: EdsBaseRef,
        inPropertyID: EdsPropertyID,
        inParam: EdsInt32,
        inPropertySize: EdsUInt32,
        outPropertyData: *mut c_void,
    ) -> EdsError;

    pub fn EdsGetPropertySize(
        inRef: EdsBaseRef,
        inPropertyID: EdsPropertyID,
        inParam: EdsInt32,
        outDataType: *mut EdsUInt32,
        outSize: *mut EdsUInt32,
    ) -> EdsError;

    pub fn EdsSetPropertyData(
        inRef: EdsBaseRef,
        inPropertyID: EdsPropertyID,
        inParam: EdsInt32,
        inPropertySize: EdsUInt32,
        inPropertyData: *const c_void,
    ) -> EdsError;

    pub fn EdsSendCommand(
        inCameraRef: EdsCameraRef,
        inCommand: EdsUInt32,
        inParam: EdsInt32,
    ) -> EdsError;

    pub fn EdsCreateMemoryStream(
        inSize: EdsUInt64,
        outStreamRef: *mut EdsStreamRef,
    ) -> EdsError;

    pub fn EdsCreateEvfImageRef(
        inStreamRef: EdsStreamRef,
        outEvfImageRef: *mut EdsEvfImageRef,
    ) -> EdsError;

    pub fn EdsDownloadEvfImage(
        inCameraRef: EdsCameraRef,
        outEvfImageRef: EdsEvfImageRef,
    ) -> EdsError;

    pub fn EdsGetPointer(
        inStreamRef: EdsStreamRef,
        outPointer: *mut *mut c_void,
    ) -> EdsError;

    pub fn EdsGetLength(
        inStreamRef: EdsStreamRef,
        outLength: *mut EdsUInt64,
    ) -> EdsError;

    pub fn EdsGetEvent() -> EdsError;
}
