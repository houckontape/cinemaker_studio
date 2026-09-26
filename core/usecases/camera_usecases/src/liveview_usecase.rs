use camera_domain::{CameraSettings, FrameBuffer};
use camera_ports::CameraLiveView;

pub struct LiveViewUseCase<P: CameraLiveView> {
    liveview_port: P,
}

impl<P: CameraLiveView> LiveViewUseCase<P> {
    pub fn new(liveview_port: P) -> Self {
        Self { liveview_port }
    }

    pub fn start(&self, camera_id: &str, buffer: FrameBuffer) {
        self.liveview_port.start_liveview(camera_id, buffer);
    }

    pub fn stop(&self) {
        self.liveview_port.stop_liveview();
    }

    pub fn is_streaming(&self) -> bool {
        self.liveview_port.is_streaming()
    }

    pub fn get_current_settings(&self) -> Option<CameraSettings> {
        self.liveview_port.get_current_settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_domain::Frame;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    struct MockCameraLiveView {
        streaming: Arc<AtomicBool>,
    }

    impl MockCameraLiveView {
        fn new() -> Self {
            Self {
                streaming: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl CameraLiveView for MockCameraLiveView {
        fn start_liveview(&self, _camera_id: &str, buffer: FrameBuffer) {
            self.streaming.store(true, Ordering::SeqCst);
            if let Ok(mut lock) = buffer.lock() {
                *lock = Some(Frame {
                    data: vec![255; 640 * 480 * 4],
                    width: 640,
                    height: 480,
                });
            }
        }

        fn stop_liveview(&self) {
            self.streaming.store(false, Ordering::SeqCst);
        }

        fn is_streaming(&self) -> bool {
            self.streaming.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_liveview_usecase_start_and_stop() {
        let mock = MockCameraLiveView::new();
        let usecase = LiveViewUseCase::new(mock);
        let buffer: FrameBuffer = Arc::new(Mutex::new(None));

        assert!(!usecase.is_streaming());
        assert!(buffer.lock().unwrap().is_none());

        usecase.start("cam_0", Arc::clone(&buffer));
        assert!(usecase.is_streaming());
        assert!(buffer.lock().unwrap().is_some());

        usecase.stop();
        assert!(!usecase.is_streaming());
    }

    #[test]
    fn test_liveview_usecase_with_ref() {
        let mock = MockCameraLiveView::new();
        let usecase = LiveViewUseCase::new(&mock);
        let buffer: FrameBuffer = Arc::new(Mutex::new(None));

        usecase.start("cam_1", Arc::clone(&buffer));
        assert!(usecase.is_streaming());
        usecase.stop();
        assert!(!usecase.is_streaming());
    }
}
