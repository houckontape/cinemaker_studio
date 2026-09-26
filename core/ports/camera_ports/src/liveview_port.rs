use camera_domain::{CameraSettings, FrameBuffer};

pub trait CameraLiveView: Send + Sync {
    fn start_liveview(&self, camera_id: &str, buffer: FrameBuffer);
    fn stop_liveview(&self);
    fn is_streaming(&self) -> bool;
    fn get_current_settings(&self) -> Option<CameraSettings> {
        None
    }
}

impl<T: CameraLiveView + ?Sized> CameraLiveView for &T {
    fn start_liveview(&self, camera_id: &str, buffer: FrameBuffer) {
        (**self).start_liveview(camera_id, buffer);
    }

    fn stop_liveview(&self) {
        (**self).stop_liveview();
    }

    fn is_streaming(&self) -> bool {
        (**self).is_streaming()
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        (**self).get_current_settings()
    }
}

impl<T: CameraLiveView + ?Sized> CameraLiveView for Box<T> {
    fn start_liveview(&self, camera_id: &str, buffer: FrameBuffer) {
        (**self).start_liveview(camera_id, buffer);
    }

    fn stop_liveview(&self) {
        (**self).stop_liveview();
    }

    fn is_streaming(&self) -> bool {
        (**self).is_streaming()
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        (**self).get_current_settings()
    }
}

impl<T: CameraLiveView + ?Sized> CameraLiveView for std::sync::Arc<T> {
    fn start_liveview(&self, camera_id: &str, buffer: FrameBuffer) {
        (**self).start_liveview(camera_id, buffer);
    }

    fn stop_liveview(&self) {
        (**self).stop_liveview();
    }

    fn is_streaming(&self) -> bool {
        (**self).is_streaming()
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        (**self).get_current_settings()
    }
}
