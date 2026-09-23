use camera_domain::{Camera, CameraKind};
use camera_ports::CameraPort;

pub struct DetectCameraUseCase<P: CameraPort> {
    camera_port: P,
}

impl<P: CameraPort> DetectCameraUseCase<P> {
    pub fn new(camera_port: P) -> Self {
        Self { camera_port }
    }

    pub fn execute(&self) -> Option<Camera> {
        let cameras = self.camera_port.list_cameras();

        // Priorité à la caméra interne (live video feed)
        if let Some(internal_cam) = cameras.iter().find(|c| c.kind == CameraKind::Internal) {
            return Some(internal_cam.clone());
        }

        // Sinon, détection des caméras USB (première caméra USB)
        if let Some(usb_cam) = cameras.iter().find(|c| c.kind == CameraKind::Usb) {
            return Some(usb_cam.clone());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCameraPort {
        cameras: Vec<Camera>,
    }

    impl CameraPort for MockCameraPort {
        fn list_cameras(&self) -> Vec<Camera> {
            self.cameras.clone()
        }
    }

    #[test]
    fn test_detect_no_camera() {
        let mock = MockCameraPort { cameras: vec![] };
        let usecase = DetectCameraUseCase::new(mock);
        assert_eq!(usecase.execute(), None);
    }

    #[test]
    fn test_detect_internal_camera_priority() {
        let mock = MockCameraPort {
            cameras: vec![
                Camera::new("usb_1", "USB Camera 1", CameraKind::Usb),
                Camera::new("cam_internal", "Integrated Webcam", CameraKind::Internal),
                Camera::new("usb_2", "USB Camera 2", CameraKind::Usb),
            ],
        };
        let usecase = DetectCameraUseCase::new(mock);
        let detected = usecase.execute().expect("Should detect a camera");
        assert_eq!(detected.id, "cam_internal");
        assert_eq!(detected.kind, CameraKind::Internal);
        assert_eq!(detected.name, "Integrated Webcam");
    }

    #[test]
    fn test_detect_first_usb_camera_when_no_internal() {
        let mock = MockCameraPort {
            cameras: vec![
                Camera::new("usb_1", "USB Camera 1", CameraKind::Usb),
                Camera::new("usb_2", "USB Camera 2", CameraKind::Usb),
            ],
        };
        let usecase = DetectCameraUseCase::new(mock);
        let detected = usecase.execute().expect("Should detect a camera");
        assert_eq!(detected.id, "usb_1");
        assert_eq!(detected.kind, CameraKind::Usb);
        assert_eq!(detected.name, "USB Camera 1");
    }

    #[test]
    fn test_detect_with_reference() {
        let mock = MockCameraPort {
            cameras: vec![Camera::new("cam_int", "Built-in", CameraKind::Internal)],
        };
        let usecase = DetectCameraUseCase::new(&mock);
        let detected = usecase.execute().expect("Should detect a camera");
        assert_eq!(detected.id, "cam_int");
    }
}
