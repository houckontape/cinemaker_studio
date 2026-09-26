use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use log::{error, info, warn};

use camera_domain::{CameraSettings, Frame, FrameBuffer};
use camera_ports::{CameraLiveView, CameraStream};

use crate::canon_adapter::CanonSdk;
use crate::edsdk_sys::*;
use crate::property_decoder::read_camera_settings;

pub struct CanonLiveViewAdapter {
    is_running: Arc<AtomicBool>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    latest_frame: Arc<Mutex<Option<Frame>>>,
    settings: Arc<Mutex<CameraSettings>>,
}

impl Default for CanonLiveViewAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonLiveViewAdapter {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            thread_handle: Mutex::new(None),
            latest_frame: Arc::new(Mutex::new(None)),
            settings: Arc::new(Mutex::new(CameraSettings::default())),
        }
    }

    pub fn get_settings(&self) -> CameraSettings {
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn parse_camera_index(camera_id: &str) -> usize {
        if let Some(stripped) = camera_id.strip_prefix("canon_") {
            stripped.parse::<usize>().unwrap_or(0)
        } else if let Some(stripped) = camera_id.strip_prefix("canon:") {
            stripped.parse::<usize>().unwrap_or(0)
        } else {
            camera_id.parse::<usize>().unwrap_or(0)
        }
    }

    fn run_liveview_loop(
        camera_index: usize,
        is_running: Arc<AtomicBool>,
        frame_buffer: Option<FrameBuffer>,
        latest_frame: Arc<Mutex<Option<Frame>>>,
        settings_store: Arc<Mutex<CameraSettings>>,
    ) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        if let Err(e) = CanonSdk::init() {
            error!("Erreur CanonSdk::init dans le thread liveview: {}", e);
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        let err = unsafe { EdsGetCameraList(&mut camera_list) };
        if err != EDS_ERR_OK || camera_list.is_null() {
            error!("Erreur EdsGetCameraList: {:#x}", err);
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
        let err = unsafe { EdsGetChildAtIndex(camera_list, camera_index as EdsInt32, &mut cam_ref) };
        if err != EDS_ERR_OK || cam_ref.is_null() {
            error!("Impossible d'obtenir la caméra Canon à l'index {}: {:#x}", camera_index, err);
            unsafe { EdsRelease(camera_list) };
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        let err = unsafe { EdsOpenSession(cam_ref) };
        if err != EDS_ERR_OK && err != EDS_ERR_SESSION_ALREADY_OPEN {
            error!("Erreur EdsOpenSession: {:#x}", err);
            unsafe {
                EdsRelease(cam_ref);
                EdsRelease(camera_list);
            }
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        // Process pending events
        unsafe {
            let _ = EdsGetEvent();
        }

        // Enable EVF mode
        let evf_mode: EdsUInt32 = kEdsEvfMode_Enable;
        let _ = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_Mode,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &evf_mode as *const _ as *const _,
            )
        };

        // Configure EVF Zoom to 1x (Fit) for lightweight liveview stream
        let evf_zoom: EdsUInt32 = kEdsEvfZoom_Fit;
        let _ = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_Zoom,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &evf_zoom as *const _ as *const _,
            )
        };

        // Set output device to PC (and keep TFT active if supported)
        let mut device: EdsUInt32 = 0;
        let _ = unsafe {
            EdsGetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &mut device as *mut _ as *mut _,
            )
        };

        device |= kEdsEvfOutputDevice_PC;
        let set_dev_err = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &device as *const _ as *const _,
            )
        };

        // Fallback: If combined device flags failed, try pure PC output
        if set_dev_err != EDS_ERR_OK {
            let pc_only: EdsUInt32 = kEdsEvfOutputDevice_PC;
            let _ = unsafe {
                EdsSetPropertyData(
                    cam_ref,
                    kEdsPropID_Evf_OutputDevice,
                    0,
                    std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                    &pc_only as *const _ as *const _,
                )
            };
        }

        // Process events and read initial default camera settings
        for _ in 0..5 {
            unsafe {
                let _ = EdsGetEvent();
            }
            thread::sleep(Duration::from_millis(20));
        }

        let initial_settings = unsafe { read_camera_settings(cam_ref) };
        if let Ok(mut lock) = settings_store.lock() {
            *lock = initial_settings;
        }

        // Allow sensor/mirror to settle for liveview
        thread::sleep(Duration::from_millis(100));

        info!("Canon LiveView démarré avec succès pour l'index {}", camera_index);

        let mut frame_count: u64 = 0;

        while is_running.load(Ordering::SeqCst) {
            unsafe {
                let _ = EdsGetEvent();
            }

            let mut stream_ref: EdsStreamRef = std::ptr::null_mut();
            let err = unsafe { EdsCreateMemoryStream(0, &mut stream_ref) };
            if err == EDS_ERR_OK && !stream_ref.is_null() {
                let mut evf_image_ref: EdsEvfImageRef = std::ptr::null_mut();
                let err = unsafe { EdsCreateEvfImageRef(stream_ref, &mut evf_image_ref) };
                if err == EDS_ERR_OK && !evf_image_ref.is_null() {
                    let dl_err = unsafe { EdsDownloadEvfImage(cam_ref, evf_image_ref) };
                    if dl_err == EDS_ERR_OK {
                        let mut ptr: *mut std::os::raw::c_void = std::ptr::null_mut();
                        let mut len: EdsUInt64 = 0;

                        let ptr_err = unsafe { EdsGetPointer(stream_ref, &mut ptr) };
                        let len_err = unsafe { EdsGetLength(stream_ref, &mut len) };

                        if ptr_err == EDS_ERR_OK && len_err == EDS_ERR_OK && !ptr.is_null() && len > 0 {
                            let jpeg_data = unsafe {
                                std::slice::from_raw_parts(ptr as *const u8, len as usize)
                            };

                            if let Ok(img) = image::load_from_memory(jpeg_data) {
                                let rgba = img.to_rgba8();
                                let width = rgba.width();
                                let height = rgba.height();
                                let raw = rgba.into_raw();

                                if let Some(ref fb) = frame_buffer
                                    && let Ok(mut lock) = fb.lock()
                                {
                                    *lock = Some(Frame {
                                        data: raw.clone(),
                                        width,
                                        height,
                                    });
                                }

                                if let Ok(mut lock) = latest_frame.lock() {
                                    *lock = Some(Frame {
                                        data: raw,
                                        width,
                                        height,
                                    });
                                }

                                frame_count += 1;
                                // Periodic settings refresh every ~30 frames (~1s)
                                if frame_count.is_multiple_of(30) {
                                    let current_settings = unsafe { read_camera_settings(cam_ref) };
                                    if let Ok(mut lock) = settings_store.lock() {
                                        *lock = current_settings;
                                    }
                                }
                            }
                        }
                    } else if dl_err == EDS_ERR_OBJECT_NOTREADY || dl_err == EDS_ERR_DEVICE_BUSY {
                        thread::sleep(Duration::from_millis(20));
                    } else {
                        warn!("EdsDownloadEvfImage code: {:#x}", dl_err);
                        thread::sleep(Duration::from_millis(30));
                    }

                    unsafe { EdsRelease(evf_image_ref) };
                }

                unsafe { EdsRelease(stream_ref) };
            }

            thread::sleep(Duration::from_millis(16));
        }

        // Disable EVF PC output
        let mut device: EdsUInt32 = 0;
        let _ = unsafe {
            EdsGetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &mut device as *mut _ as *mut _,
            )
        };
        device &= !kEdsEvfOutputDevice_PC;
        let _ = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &device as *const _ as *const _,
            )
        };

        unsafe {
            EdsCloseSession(cam_ref);
            EdsRelease(cam_ref);
            EdsRelease(camera_list);
        }

        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Com::CoUninitialize;
            CoUninitialize();
        }

        info!("Canon LiveView arrêté.");
    }
}

impl CameraLiveView for CanonLiveViewAdapter {
    fn start_liveview(&self, camera_id: &str, frame_buffer: FrameBuffer) {
        self.stop_liveview();

        let camera_index = Self::parse_camera_index(camera_id);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let latest_frame = Arc::clone(&self.latest_frame);
        let settings_store = Arc::clone(&self.settings);

        let handle = thread::spawn(move || {
            Self::run_liveview_loop(
                camera_index,
                is_running,
                Some(frame_buffer),
                latest_frame,
                settings_store,
            );
        });

        if let Ok(mut lock) = self.thread_handle.lock() {
            *lock = Some(handle);
        }
    }

    fn stop_liveview(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            let handle = if let Ok(mut lock) = self.thread_handle.lock() {
                lock.take()
            } else {
                None
            };
            if let Some(handle) = handle {
                let _ = handle.join();
            }
        }
    }

    fn is_streaming(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        let s = self.get_settings();
        if !s.is_empty() {
            Some(s)
        } else {
            None
        }
    }
}

impl CameraStream for CanonLiveViewAdapter {
    fn start_stream(&self, camera_id: &str) {
        if self.is_streaming() {
            self.stop_stream();
        }

        let camera_index = Self::parse_camera_index(camera_id);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let latest_frame = Arc::clone(&self.latest_frame);
        let settings_store = Arc::clone(&self.settings);

        let handle = thread::spawn(move || {
            Self::run_liveview_loop(
                camera_index,
                is_running,
                None,
                latest_frame,
                settings_store,
            );
        });

        if let Ok(mut lock) = self.thread_handle.lock() {
            *lock = Some(handle);
        }
    }

    fn get_frame(&self) -> Option<Frame> {
        if let Ok(guard) = self.latest_frame.lock() {
            guard.clone()
        } else {
            None
        }
    }

    fn stop_stream(&self) {
        self.stop_liveview();
    }
}

pub type LiveViewStream = CanonLiveViewAdapter;
