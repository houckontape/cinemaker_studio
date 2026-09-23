use std::os::raw::{c_int, c_uint, c_void};

pub type EdsError = c_int;
pub type EdsUInt32 = c_uint;
pub type EdsInt32 = c_int;
pub type EdsBool = c_int;

#[repr(C)]
pub struct EdsBaseRef {
    pub ptr: *mut c_void,
}

pub type EdsCameraRef = *mut EdsBaseRef;
pub type EdsCameraListRef = *mut EdsBaseRef;
pub type EdsEvfImageRef = *mut EdsBaseRef;
pub type EdsStreamRef = *mut EdsBaseRef;

extern "C" {
    pub fn EdsInitializeSDK() -> EdsError;
    pub fn EdsTerminateSDK() -> EdsError;

    pub fn EdsGetCameraList(out_camera_list: *mut EdsCameraListRef) -> EdsError;
    pub fn EdsGetChildCount(in_ref: EdsCameraListRef, out_count: *mut EdsUInt32) -> EdsError;
    pub fn EdsGetChildAtIndex(
        in_ref: EdsCameraListRef,
        index: EdsInt32,
        out_camera_ref: *mut EdsCameraRef,
    ) -> EdsError;

    pub fn EdsOpenSession(in_camera_ref: EdsCameraRef) -> EdsError;
    pub fn EdsCloseSession(in_camera_ref: EdsCameraRef) -> EdsError;

    pub fn EdsRelease(in_ref: EdsBaseRef) -> EdsError;

    pub fn EdsGetPropertyData(
        in_ref: EdsBaseRef,
        in_property_id: EdsUInt32,
        in_param: EdsInt32,
        in_size: EdsUInt32,
        out_data: *mut c_void,
    ) -> EdsError;

    pub fn EdsSetPropertyData(
        in_ref: EdsBaseRef,
        in_property_id: EdsUInt32,
        in_param: EdsInt32,
        in_size: EdsUInt32,
        in_data: *const c_void,
    ) -> EdsError;

    pub fn EdsSendCommand(
        in_camera_ref: EdsCameraRef,
        in_command: EdsUInt32,
        in_param: EdsUInt32,
    ) -> EdsError;

    pub fn EdsCreateEvfImageRef(
        in_stream_ref: EdsStreamRef,
        out_evf_image_ref: *mut EdsEvfImageRef,
    ) -> EdsError;

    pub fn EdsDownloadEvfImage(
        in_camera_ref: EdsCameraRef,
        out_evf_image_ref: EdsEvfImageRef,
    ) -> EdsError;
}

// Quelques IDs courants (à compléter selon EDSDK)
pub const K_EDS_PROP_ID_PRODUCT_NAME: EdsUInt32 = 0x00000002;
pub const K_EDS_PROP_ID_EVF_OUTPUT_DEVICE: EdsUInt32 = 0x00000500;
pub const K_EDS_PROP_ID_EVF_DATA: EdsUInt32 = 0x00000501;

pub const K_EDS_EVF_OUTPUT_DEVICE_PC: EdsUInt32 = 0x00000002;

pub const K_EDS_CAMERA_COMMAND_PRESS_SHUTTER_BUTTON: EdsUInt32 = 0x00000004;
pub const K_EDS_CAMERA_COMMAND_SHUTTER_BUTTON_COMPLETELY: EdsUInt32 = 0x00000002;
pub const K_EDS_CAMERA_COMMAND_SHUTTER_BUTTON_OFF: EdsUInt32 = 0x00000000;
