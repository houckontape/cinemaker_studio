use camera_adapter_canon::{CanonCameraAdapter, CanonLiveViewAdapter};
use camera_adapter_windows::MediaFoundationLiveViewAdapter;
use camera_domain::{Camera, CameraKind, CameraSettings, FrameBuffer};
use camera_ports::{CameraLiveView, CameraPort};
use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;

#[derive(Clone, Default)]
pub struct CameraAdapter;

impl CameraAdapter {
    pub fn new() -> Self {
        unsafe {
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        Self
    }

    fn list_windows_cameras(&self) -> Vec<Camera> {
        unsafe {
            let mut attributes: Option<IMFAttributes> = None;
            if MFCreateAttributes(&mut attributes, 1).is_err() {
                return Vec::new();
            }
            let Some(attributes) = attributes else {
                return Vec::new();
            };

            if attributes
                .SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .is_err()
            {
                return Vec::new();
            }

            let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
            let mut count = 0;

            if MFEnumDeviceSources(&attributes, &mut devices, &mut count).is_err() {
                return Vec::new();
            }

            if devices.is_null() || count == 0 {
                return Vec::new();
            }

            let slice = std::slice::from_raw_parts(devices, count as usize);
            let mut cameras = Vec::new();

            for (i, dev_opt) in slice.iter().enumerate() {
                if let Some(dev) = dev_opt {
                    let mut name_ptr = PWSTR::null();
                    let mut len = 0;
                    let name = if dev
                        .GetAllocatedString(&MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &mut name_ptr, &mut len)
                        .is_ok()
                    {
                        let s = String::from_utf16_lossy(std::slice::from_raw_parts(name_ptr.0, len as usize));
                        CoTaskMemFree(Some(name_ptr.0 as _));
                        s
                    } else {
                        format!("Camera {}", i)
                    };

                    let mut sym_ptr = PWSTR::null();
                    let mut sym_len = 0;
                    let sym_link = if dev
                        .GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                            &mut sym_ptr,
                            &mut sym_len,
                        )
                        .is_ok()
                    {
                        let s = String::from_utf16_lossy(std::slice::from_raw_parts(sym_ptr.0, sym_len as usize));
                        CoTaskMemFree(Some(sym_ptr.0 as _));
                        s
                    } else {
                        String::new()
                    };

                    let name_lower = name.to_lowercase();
                    let sym_lower = sym_link.to_lowercase();

                    let kind = if name_lower.contains("internal")
                        || name_lower.contains("integrated")
                        || name_lower.contains("built-in")
                        || name_lower.contains("builtin")
                        || name_lower.contains("facetime")
                        || name_lower.contains("webcam")
                        || name_lower.contains("easycamera")
                        || sym_lower.contains("internal")
                        || sym_lower.contains("integrated")
                        || sym_lower.contains("acpi")
                    {
                        CameraKind::Internal
                    } else {
                        CameraKind::Usb
                    };

                    cameras.push(Camera::new(i.to_string(), name, kind));
                }
            }

            CoTaskMemFree(Some(devices as _));

            cameras
        }
    }
}

impl CameraPort for CameraAdapter {
    fn list_cameras(&self) -> Vec<Camera> {
        let mut cameras = Vec::new();

        // 1. Détection prioritaire des caméras Canon (USB via EDSDK)
        let canon_adapter = CanonCameraAdapter::new();
        let canon_cameras = canon_adapter.list_cameras();
        cameras.extend(canon_cameras);

        // 2. Détection des caméras Windows Media Foundation
        let mf_cameras = self.list_windows_cameras();
        cameras.extend(mf_cameras);

        cameras
    }

    fn get_camera_settings(&self, camera_id: &str) -> Option<CameraSettings> {
        if camera_id.starts_with("canon") {
            let canon_adapter = CanonCameraAdapter::new();
            canon_adapter.get_camera_settings(camera_id)
        } else {
            None
        }
    }
}

pub struct UnifiedLiveViewAdapter {
    canon: CanonLiveViewAdapter,
    windows: MediaFoundationLiveViewAdapter,
}

impl Default for UnifiedLiveViewAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedLiveViewAdapter {
    pub fn new() -> Self {
        Self {
            canon: CanonLiveViewAdapter::new(),
            windows: MediaFoundationLiveViewAdapter::new(),
        }
    }
}

impl CameraLiveView for UnifiedLiveViewAdapter {
    fn start_liveview(&self, camera_id: &str, frame_buffer: FrameBuffer) {
        self.stop_liveview();
        if camera_id.starts_with("canon") {
            self.canon.start_liveview(camera_id, frame_buffer);
        } else {
            self.windows.start_liveview(camera_id, frame_buffer);
        }
    }

    fn stop_liveview(&self) {
        self.canon.stop_liveview();
        self.windows.stop_liveview();
    }

    fn is_streaming(&self) -> bool {
        self.canon.is_streaming() || self.windows.is_streaming()
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        self.canon.get_current_settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_hardware_cameras() {
        let adapter = CameraAdapter::new();
        let cameras = adapter.list_cameras();
        println!("Detected cameras: {:#?}", cameras);
    }

    #[test]
    fn test_workspace_selection_and_liveview_control() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use camera_domain::CameraKind;

        struct MockCameraPort {
            cams: Vec<Camera>,
        }
        impl CameraPort for MockCameraPort {
            fn list_cameras(&self) -> Vec<Camera> {
                self.cams.clone()
            }
        }

        struct MockLiveView {
            streaming: Arc<AtomicBool>,
        }
        impl CameraLiveView for MockLiveView {
            fn start_liveview(&self, _camera_id: &str, _frame_buffer: FrameBuffer) {
                self.streaming.store(true, Ordering::SeqCst);
            }
            fn stop_liveview(&self) {
                self.streaming.store(false, Ordering::SeqCst);
            }
            fn is_streaming(&self) -> bool {
                self.streaming.load(Ordering::SeqCst)
            }
        }

        let streaming_flag = Arc::new(AtomicBool::new(false));
        let mock_lv = MockLiveView {
            streaming: Arc::clone(&streaming_flag),
        };
        let mock_port = MockCameraPort {
            cams: vec![
                Camera::new("canon_0", "Canon EOS R5", CameraKind::Usb),
                Camera::new("0", "Integrated Webcam", CameraKind::Internal),
            ],
        };

        let mut app = crate::workspace_app::WorkspaceApp::new_with_ports(mock_port, mock_lv);

        assert_eq!(app.available_cameras().len(), 2);
        assert_eq!(app.selected_camera_id(), Some("canon_0"));
        assert!(!app.is_streaming());

        // Test start liveview
        app.start_liveview();
        assert!(app.is_streaming());

        // Test toggle liveview
        app.toggle_liveview();
        assert!(!app.is_streaming());

        // Test change selection
        app.start_liveview();
        assert!(app.is_streaming());
        app.refresh_camera();
        assert_eq!(app.selected_camera_id(), Some("canon_0"));
    }
}
