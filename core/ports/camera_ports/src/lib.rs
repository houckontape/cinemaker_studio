pub mod camera_port;
pub mod liveview_port;
pub use camera_port::*;
pub use liveview_port::*;

use camera_domain::{CameraInfo, Frame};

pub trait CameraDetector {
    fn list_cameras(&self) -> Vec<CameraInfo>;
}

pub trait CameraStream {
    fn start_stream(&self, camera_id: &str);
    fn get_frame(&self) -> Option<Frame>;
    fn stop_stream(&self);
}

impl<T: CameraDetector + ?Sized> CameraDetector for &T {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        (**self).list_cameras()
    }
}

impl<T: CameraDetector + ?Sized> CameraDetector for Box<T> {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        (**self).list_cameras()
    }
}

impl<T: CameraDetector + ?Sized> CameraDetector for std::sync::Arc<T> {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        (**self).list_cameras()
    }
}

impl<T: CameraStream + ?Sized> CameraStream for &T {
    fn start_stream(&self, camera_id: &str) {
        (**self).start_stream(camera_id);
    }

    fn get_frame(&self) -> Option<Frame> {
        (**self).get_frame()
    }

    fn stop_stream(&self) {
        (**self).stop_stream();
    }
}

impl<T: CameraStream + ?Sized> CameraStream for Box<T> {
    fn start_stream(&self, camera_id: &str) {
        (**self).start_stream(camera_id);
    }

    fn get_frame(&self) -> Option<Frame> {
        (**self).get_frame()
    }

    fn stop_stream(&self) {
        (**self).stop_stream();
    }
}

impl<T: CameraStream + ?Sized> CameraStream for std::sync::Arc<T> {
    fn start_stream(&self, camera_id: &str) {
        (**self).start_stream(camera_id);
    }

    fn get_frame(&self) -> Option<Frame> {
        (**self).get_frame()
    }

    fn stop_stream(&self) {
        (**self).stop_stream();
    }
}
