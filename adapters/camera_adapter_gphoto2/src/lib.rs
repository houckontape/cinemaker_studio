use std::sync::Mutex;
use camera_domain::{CameraInfo, Frame};
use camera_ports::{CameraDetector, CameraStream};
use gphoto2::{Camera, Context};

pub struct GPhoto2Adapter {
    camera: Mutex<Option<Camera>>,
}

impl GPhoto2Adapter {
    pub fn new() -> Self {
        Self {
            camera: Mutex::new(None),
        }
    }
}

impl Default for GPhoto2Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraDetector for GPhoto2Adapter {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        let mut cameras = Vec::new();

        let Ok(context) = Context::new() else {
            return cameras;
        };

        if let Ok(camera_list) = context.list_cameras().wait() {
            for (i, desc) in camera_list.enumerate() {
                cameras.push(CameraInfo {
                    id: if desc.port.is_empty() {
                        i.to_string()
                    } else {
                        desc.port.clone()
                    },
                    name: desc.model.clone(),
                });
            }
        }

        // Si la liste est vide, tentative d'autodétection directe
        if cameras.is_empty() {
            if let Ok(camera) = context.autodetect_camera().wait() {
                let model = camera.abilities().model().to_string();
                let port = camera
                    .port_info()
                    .map(|p| p.path().to_string())
                    .unwrap_or_else(|_| "usb:".to_string());

                cameras.push(CameraInfo {
                    id: port,
                    name: model,
                });
            }
        }

        cameras
    }
}

impl CameraStream for GPhoto2Adapter {
    fn start_stream(&self, _camera_id: &str) {
        if let Ok(context) = Context::new() {
            if let Ok(camera) = context.autodetect_camera().wait() {
                if let Ok(mut lock) = self.camera.lock() {
                    *lock = Some(camera);
                }
            }
        }
    }

    fn get_frame(&self) -> Option<Frame> {
        let lock = self.camera.lock().ok()?;
        let camera = lock.as_ref()?;

        // Capture d'image via gphoto2
        if let Ok(file_path) = camera.capture_image().wait() {
            let mut file = camera.fs().download(&file_path.folder(), &file_path.name()).wait().ok()?;
            let data = file.data().to_vec();
            Some(Frame {
                data,
                width: 0,
                height: 0,
            })
        } else if let Ok(preview) = camera.capture_preview().wait() {
            let data = preview.data().to_vec();
            Some(Frame {
                data,
                width: 0,
                height: 0,
            })
        } else {
            None
        }
    }

    fn stop_stream(&self) {
        if let Ok(mut lock) = self.camera.lock() {
            *lock = None;
        }
    }
}
