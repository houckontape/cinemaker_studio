use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::os::raw::c_char;
use std::thread;
use std::time::Duration;
use log::warn;

use camera_domain::{Camera, CameraInfo, CameraKind, CameraSettings};
use camera_ports::{CameraDetector, CameraPort};

use crate::edsdk_sys::*;
use crate::error::{CanonError, CanonResult};
use crate::property_decoder::read_camera_settings;

pub static LIVEVIEW_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct CanonSdk;

static SDK_STATE: Mutex<bool> = Mutex::new(false);

impl CanonSdk {
    pub fn init() -> CanonResult<()> {
        let mut initialized = SDK_STATE.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let err = unsafe { EdsInitializeSDK() };
        if err == EDS_ERR_OK || err == 2 {
            *initialized = true;
            Ok(())
        } else {
            Err(CanonError::SdkInitFailed(err))
        }
    }

    pub fn terminate() {
        let mut initialized = SDK_STATE.lock().unwrap();
        if *initialized {
            unsafe {
                EdsTerminateSDK();
            }
            *initialized = false;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedCamera {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub port_name: String,
}

#[derive(Default)]
pub struct CanonCameraAdapter;

impl CanonCameraAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_cameras() -> CanonResult<Vec<DetectedCamera>> {
        CanonSdk::init()?;

        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        let err = unsafe { EdsGetCameraList(&mut camera_list) };
        if err != EDS_ERR_OK {
            return Err(CanonError::EdsError(err));
        }

        let mut count: EdsUInt32 = 0;
        let err = unsafe { EdsGetChildCount(camera_list, &mut count) };
        if err != EDS_ERR_OK {
            unsafe { EdsRelease(camera_list) };
            return Err(CanonError::EdsError(err));
        }

        let mut cameras = Vec::new();

        for i in 0..count {
            let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
            let err = unsafe { EdsGetChildAtIndex(camera_list, i as EdsInt32, &mut cam_ref) };
            if err != EDS_ERR_OK || cam_ref.is_null() {
                continue;
            }

            let mut device_info = EdsDeviceInfo::default();
            let err_info = unsafe { EdsGetDeviceInfo(cam_ref, &mut device_info) };

            let mut name = if err_info == EDS_ERR_OK {
                unsafe { c_char_to_string(&device_info.szDeviceDescription) }
            } else {
                String::new()
            };

            let port_name = if err_info == EDS_ERR_OK {
                unsafe { c_char_to_string(&device_info.szPortName) }
            } else {
                String::new()
            };

            // Si le nom est vide, tentative de lecture via kEdsPropID_ProductName
            if name.is_empty() {
                let mut name_buf = [0 as c_char; 256];
                let prop_err = unsafe {
                    EdsGetPropertyData(
                        cam_ref,
                        kEdsPropID_ProductName,
                        0,
                        std::mem::size_of_val(&name_buf) as EdsUInt32,
                        name_buf.as_mut_ptr() as *mut _,
                    )
                };
                if prop_err == EDS_ERR_OK {
                    name = unsafe { c_char_to_string(&name_buf) };
                }
            }

            if name.is_empty() {
                name = format!("Canon Camera {}", i + 1);
            }

            cameras.push(DetectedCamera {
                index: i as usize,
                id: format!("canon_{}", i),
                name,
                port_name,
            });

            unsafe { EdsRelease(cam_ref) };
        }

        unsafe { EdsRelease(camera_list) };

        Ok(cameras)
    }

    pub fn query_settings_for_camera(index: usize) -> CanonResult<CameraSettings> {
        CanonSdk::init()?;

        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        let err = unsafe { EdsGetCameraList(&mut camera_list) };
        if err != EDS_ERR_OK {
            return Err(CanonError::EdsError(err));
        }

        let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
        let err = unsafe { EdsGetChildAtIndex(camera_list, index as EdsInt32, &mut cam_ref) };
        if err != EDS_ERR_OK || cam_ref.is_null() {
            unsafe { EdsRelease(camera_list) };
            return Err(CanonError::CameraNotFound);
        }

        // Tentative d'ouverture de session avec gestion de retry si busy
        let mut open_err = unsafe { EdsOpenSession(cam_ref) };
        let mut retries = 0;
        while (open_err == EDS_ERR_DEVICE_BUSY || open_err == EDS_ERR_COMM_PORT_IS_IN_USE) && retries < 5 {
            thread::sleep(Duration::from_millis(50));
            unsafe {
                let _ = EdsGetEvent();
                open_err = EdsOpenSession(cam_ref);
            }
            retries += 1;
        }

        let is_liveview = LIVEVIEW_ACTIVE.load(Ordering::SeqCst);

        let settings = if open_err == EDS_ERR_OK {
            // Laisser le temps à la caméra de transmettre la table des propriétés via USB
            for _ in 0..6 {
                unsafe {
                    let _ = EdsGetEvent();
                }
                thread::sleep(Duration::from_millis(25));
            }
            let s = unsafe { read_camera_settings(cam_ref) };
            if !is_liveview {
                unsafe { EdsCloseSession(cam_ref) };
            }
            s
        } else if open_err == EDS_ERR_SESSION_ALREADY_OPEN {
            unsafe {
                let _ = EdsGetEvent();
            }
            // Ne pas fermer la session car elle est utilisée par le flux LiveView
            unsafe { read_camera_settings(cam_ref) }
        } else {
            warn!("EdsOpenSession failed in query_settings_for_camera: {:#x}", open_err);
            CameraSettings::default()
        };

        unsafe {
            EdsRelease(cam_ref);
            EdsRelease(camera_list);
        }

        Ok(settings)
    }
}

impl CameraDetector for CanonCameraAdapter {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        match Self::detect_cameras() {
            Ok(cams) => cams
                .into_iter()
                .map(|c| CameraInfo {
                    id: c.id,
                    name: c.name,
                })
                .collect(),
            Err(e) => {
                warn!("Impossible de lister les caméras Canon via EDSDK: {}", e);
                Vec::new()
            }
        }
    }
}

impl CameraPort for CanonCameraAdapter {
    fn list_cameras(&self) -> Vec<Camera> {
        match Self::detect_cameras() {
            Ok(cams) => cams
                .into_iter()
                .map(|c| Camera {
                    id: c.id,
                    name: c.name,
                    kind: CameraKind::Usb,
                })
                .collect(),
            Err(e) => {
                warn!("Impossible de lister les caméras Canon (CameraPort): {}", e);
                Vec::new()
            }
        }
    }

    fn get_camera_settings(&self, camera_id: &str) -> Option<CameraSettings> {
        if let Some(stripped) = camera_id.strip_prefix("canon_")
            && let Ok(idx) = stripped.parse::<usize>()
        {
            return Self::query_settings_for_camera(idx).ok();
        }
        None
    }
}

unsafe fn c_char_to_string(slice: &[c_char]) -> String {
    let bytes: Vec<u8> = slice
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).trim().to_string()
}
