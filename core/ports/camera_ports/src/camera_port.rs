use camera_domain::{Camera, CameraSettings};

pub trait CameraPort {
    fn list_cameras(&self) -> Vec<Camera>;
    fn get_camera_settings(&self, _camera_id: &str) -> Option<CameraSettings> {
        None
    }
}

impl<T: CameraPort + ?Sized> CameraPort for &T {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }

    fn get_camera_settings(&self, camera_id: &str) -> Option<CameraSettings> {
        (**self).get_camera_settings(camera_id)
    }
}

impl<T: CameraPort + ?Sized> CameraPort for Box<T> {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }

    fn get_camera_settings(&self, camera_id: &str) -> Option<CameraSettings> {
        (**self).get_camera_settings(camera_id)
    }
}

impl<T: CameraPort + ?Sized> CameraPort for std::sync::Arc<T> {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }

    fn get_camera_settings(&self, camera_id: &str) -> Option<CameraSettings> {
        (**self).get_camera_settings(camera_id)
    }
}
