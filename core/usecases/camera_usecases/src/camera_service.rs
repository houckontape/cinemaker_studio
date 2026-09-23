use camera_domain::CameraInfo;
use camera_ports::CameraDetector;

pub struct CameraService<I: CameraDetector, G: CameraDetector, M: CameraDetector> {
    internal_detector: I,
    gphoto2_detector: G,
    media_foundation_detector: M,
}

impl<I: CameraDetector, G: CameraDetector, M: CameraDetector> CameraService<I, G, M> {
    pub fn new(internal_detector: I, gphoto2_detector: G, media_foundation_detector: M) -> Self {
        Self {
            internal_detector,
            gphoto2_detector,
            media_foundation_detector,
        }
    }

    /// Sélectionne la meilleure caméra selon l'ordre de priorité (fallback) :
    /// 1. Caméra interne détectée -> utiliser interne
    /// 2. Sinon -> utiliser gphoto2
    /// 3. Sinon -> utiliser USB MediaFoundation
    pub fn select_camera(&self) -> Option<CameraInfo> {
        // 1. Priorité à la caméra interne
        let internal_cameras = self.internal_detector.list_cameras();
        if let Some(cam) = internal_cameras.into_iter().next() {
            return Some(cam);
        }

        // 2. Sinon, caméra gphoto2
        let gphoto2_cameras = self.gphoto2_detector.list_cameras();
        if let Some(cam) = gphoto2_cameras.into_iter().next() {
            return Some(cam);
        }

        // 3. Sinon, caméra USB MediaFoundation
        let mf_cameras = self.media_foundation_detector.list_cameras();
        if let Some(cam) = mf_cameras.into_iter().next() {
            return Some(cam);
        }

        None
    }

    /// Alias pour select_camera()
    pub fn detect_camera(&self) -> Option<CameraInfo> {
        self.select_camera()
    }

    /// Retourne toutes les caméras détectées par l'ensemble des adaptateurs
    pub fn list_all_cameras(&self) -> Vec<CameraInfo> {
        let mut cameras = Vec::new();
        cameras.extend(self.internal_detector.list_cameras());
        cameras.extend(self.gphoto2_detector.list_cameras());
        cameras.extend(self.media_foundation_detector.list_cameras());
        cameras
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDetector {
        cameras: Vec<CameraInfo>,
    }

    impl MockDetector {
        fn new(cameras: Vec<CameraInfo>) -> Self {
            Self { cameras }
        }

        fn empty() -> Self {
            Self { cameras: vec![] }
        }
    }

    impl CameraDetector for MockDetector {
        fn list_cameras(&self) -> Vec<CameraInfo> {
            self.cameras.clone()
        }
    }

    #[test]
    fn test_fallback_priority_internal() {
        let internal = MockDetector::new(vec![CameraInfo {
            id: "internal_0".to_string(),
            name: "Integrated Webcam".to_string(),
        }]);
        let gphoto2 = MockDetector::new(vec![CameraInfo {
            id: "usb:001,002".to_string(),
            name: "Canon EOS 100D".to_string(),
        }]);
        let mf = MockDetector::new(vec![CameraInfo {
            id: "0".to_string(),
            name: "USB Video Device".to_string(),
        }]);

        let service = CameraService::new(internal, gphoto2, mf);
        let selected = service.select_camera().expect("Should detect internal camera");
        assert_eq!(selected.id, "internal_0");
        assert_eq!(selected.name, "Integrated Webcam");
    }

    #[test]
    fn test_fallback_priority_gphoto2_when_no_internal() {
        let internal = MockDetector::empty();
        let gphoto2 = MockDetector::new(vec![CameraInfo {
            id: "usb:001,002".to_string(),
            name: "Nikon DSC D3400".to_string(),
        }]);
        let mf = MockDetector::new(vec![CameraInfo {
            id: "0".to_string(),
            name: "USB Video Device".to_string(),
        }]);

        let service = CameraService::new(internal, gphoto2, mf);
        let selected = service.select_camera().expect("Should detect gphoto2 camera");
        assert_eq!(selected.id, "usb:001,002");
        assert_eq!(selected.name, "Nikon DSC D3400");
    }

    #[test]
    fn test_fallback_priority_media_foundation_when_no_internal_and_no_gphoto2() {
        let internal = MockDetector::empty();
        let gphoto2 = MockDetector::empty();
        let mf = MockDetector::new(vec![CameraInfo {
            id: "0".to_string(),
            name: "USB Video Device".to_string(),
        }]);

        let service = CameraService::new(internal, gphoto2, mf);
        let selected = service.select_camera().expect("Should detect MF camera");
        assert_eq!(selected.id, "0");
        assert_eq!(selected.name, "USB Video Device");
    }

    #[test]
    fn test_fallback_when_no_camera_available() {
        let internal = MockDetector::empty();
        let gphoto2 = MockDetector::empty();
        let mf = MockDetector::empty();

        let service = CameraService::new(internal, gphoto2, mf);
        assert_eq!(service.select_camera(), None);
    }

    #[test]
    fn test_list_all_cameras() {
        let internal = MockDetector::new(vec![CameraInfo {
            id: "int_0".to_string(),
            name: "Internal Cam".to_string(),
        }]);
        let gphoto2 = MockDetector::new(vec![CameraInfo {
            id: "usb:001,002".to_string(),
            name: "Sony Alpha".to_string(),
        }]);
        let mf = MockDetector::new(vec![CameraInfo {
            id: "0".to_string(),
            name: "USB Cam".to_string(),
        }]);

        let service = CameraService::new(internal, gphoto2, mf);
        let all = service.list_all_cameras();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].name, "Internal Cam");
        assert_eq!(all[1].name, "Sony Alpha");
        assert_eq!(all[2].name, "USB Cam");
    }
}
