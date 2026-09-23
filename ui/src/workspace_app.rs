use camera_domain::{Camera, FrameBuffer};
use camera_ports::{CameraLiveView, CameraPort};
use camera_usecases::{DetectCameraUseCase, LiveViewUseCase};
use crate::adapters::camera_adapter::{CameraAdapter, UnifiedLiveViewAdapter};
use crate::components::camera_panel::CameraPanel;
use crate::liveview_panel::LiveViewPanel;
use eframe::App;
use egui::{TextureHandle, Ui};
use std::sync::{Arc, Mutex};

pub struct WorkspaceApp<P: CameraPort = CameraAdapter, L: CameraLiveView = UnifiedLiveViewAdapter> {
    camera_port: P,
    liveview_usecase: LiveViewUseCase<L>,
    detected_camera: Option<Camera>,
    frame_buffer: FrameBuffer,
    texture: Option<TextureHandle>,
}

impl Default for WorkspaceApp<CameraAdapter, UnifiedLiveViewAdapter> {
    fn default() -> Self {
        let adapter = CameraAdapter::new();
        let liveview_adapter = UnifiedLiveViewAdapter::new();
        Self::new_with_ports(adapter, liveview_adapter)
    }
}

impl<P: CameraPort, L: CameraLiveView> WorkspaceApp<P, L> {
    pub fn new_with_ports(camera_port: P, liveview_port: L) -> Self {
        let detect_usecase = DetectCameraUseCase::new(&camera_port);
        let detected_camera = detect_usecase.execute();
        let liveview_usecase = LiveViewUseCase::new(liveview_port);
        let frame_buffer: FrameBuffer = Arc::new(Mutex::new(None));

        if let Some(ref camera) = detected_camera {
            liveview_usecase.start(&camera.id, Arc::clone(&frame_buffer));
        }

        Self {
            camera_port,
            liveview_usecase,
            detected_camera,
            frame_buffer,
            texture: None,
        }
    }

    pub fn new(camera_port: P) -> WorkspaceApp<P, UnifiedLiveViewAdapter> {
        let liveview_adapter = UnifiedLiveViewAdapter::new();
        WorkspaceApp::new_with_ports(camera_port, liveview_adapter)
    }

    pub fn refresh_camera(&mut self) {
        let usecase = DetectCameraUseCase::new(&self.camera_port);
        let new_camera = usecase.execute();

        if new_camera != self.detected_camera {
            self.liveview_usecase.stop();
            if let Ok(mut lock) = self.frame_buffer.lock() {
                *lock = None;
            }
            if let Some(ref camera) = new_camera {
                self.liveview_usecase.start(&camera.id, Arc::clone(&self.frame_buffer));
            }
            self.detected_camera = new_camera;
        }
    }

    pub fn detected_camera(&self) -> Option<&Camera> {
        self.detected_camera.as_ref()
    }

    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    pub fn is_streaming(&self) -> bool {
        self.liveview_usecase.is_streaming()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Dragonframe Light - Workspace");
        ui.separator();

        let mut refresh_requested = false;
        CameraPanel::show(ui, &self.detected_camera, || {
            refresh_requested = true;
        });

        if refresh_requested {
            self.refresh_camera();
        }

        ui.separator();

        LiveViewPanel::show(
            ui,
            &self.frame_buffer,
            &mut self.texture,
            self.liveview_usecase.is_streaming(),
        );

        if self.liveview_usecase.is_streaming() {
            ui.ctx().request_repaint();
        }
    }
}

impl<P: CameraPort, L: CameraLiveView> App for WorkspaceApp<P, L> {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.ui(ui);
    }
}
