use camera_domain::Camera;

pub trait CameraPort {
    fn list_cameras(&self) -> Vec<Camera>;
}

impl<T: CameraPort + ?Sized> CameraPort for &T {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }
}

impl<T: CameraPort + ?Sized> CameraPort for Box<T> {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }
}

impl<T: CameraPort + ?Sized> CameraPort for std::sync::Arc<T> {
    fn list_cameras(&self) -> Vec<Camera> {
        (**self).list_cameras()
    }
}
