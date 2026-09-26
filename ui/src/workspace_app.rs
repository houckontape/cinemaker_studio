use camera_domain::{Camera, CameraSettings, FrameBuffer};
use camera_ports::{CameraLiveView, CameraPort};
use camera_usecases::LiveViewUseCase;
use crate::adapters::camera_adapter::{CameraAdapter, UnifiedLiveViewAdapter};
use crate::components::camera_panel::CameraPanel;
use crate::liveview_panel::LiveViewPanel;
use eframe::App;
use egui::{TextureHandle, Ui};
use std::sync::{Arc, Mutex};

pub struct WorkspaceApp<P: CameraPort = CameraAdapter, L: CameraLiveView = UnifiedLiveViewAdapter> {
    camera_port: P,
    liveview_usecase: LiveViewUseCase<L>,
    available_cameras: Vec<Camera>,
    selected_camera_id: Option<String>,
    current_settings: Option<CameraSettings>,
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
        let available_cameras = camera_port.list_cameras();
        let selected_camera_id = available_cameras.first().map(|c| c.id.clone());
        let current_settings = selected_camera_id
            .as_deref()
            .and_then(|id| camera_port.get_camera_settings(id));
        let liveview_usecase = LiveViewUseCase::new(liveview_port);
        let frame_buffer: FrameBuffer = Arc::new(Mutex::new(None));

        Self {
            camera_port,
            liveview_usecase,
            available_cameras,
            selected_camera_id,
            current_settings,
            frame_buffer,
            texture: None,
        }
    }

    pub fn new(camera_port: P) -> WorkspaceApp<P, UnifiedLiveViewAdapter> {
        let liveview_adapter = UnifiedLiveViewAdapter::new();
        WorkspaceApp::new_with_ports(camera_port, liveview_adapter)
    }

    pub fn refresh_camera(&mut self) {
        let new_cameras = self.camera_port.list_cameras();

        // Si la caméra précédemment sélectionnée n'existe plus, on sélectionne la première disponible
        if let Some(ref selected_id) = self.selected_camera_id {
            if !new_cameras.iter().any(|c| &c.id == selected_id) {
                if self.liveview_usecase.is_streaming() {
                    self.stop_liveview();
                }
                self.selected_camera_id = new_cameras.first().map(|c| c.id.clone());
            }
        } else {
            self.selected_camera_id = new_cameras.first().map(|c| c.id.clone());
        }

        self.available_cameras = new_cameras;
        self.refresh_settings();
    }

    pub fn refresh_settings(&mut self) {
        if let Some(settings) = self.liveview_usecase.get_current_settings() {
            self.current_settings = Some(settings);
        } else if let Some(ref selected_id) = self.selected_camera_id {
            self.current_settings = self.camera_port.get_camera_settings(selected_id);
        } else {
            self.current_settings = None;
        }
    }

    pub fn start_liveview(&mut self) {
        if let Some(ref camera_id) = self.selected_camera_id {
            self.liveview_usecase.stop();
            if let Ok(mut lock) = self.frame_buffer.lock() {
                *lock = None;
            }
            self.liveview_usecase.start(camera_id, Arc::clone(&self.frame_buffer));
            self.refresh_settings();
        }
    }

    pub fn stop_liveview(&mut self) {
        self.liveview_usecase.stop();
        if let Ok(mut lock) = self.frame_buffer.lock() {
            *lock = None;
        }
    }

    pub fn toggle_liveview(&mut self) {
        if self.liveview_usecase.is_streaming() {
            self.stop_liveview();
        } else {
            self.start_liveview();
        }
    }

    pub fn available_cameras(&self) -> &[Camera] {
        &self.available_cameras
    }

    pub fn selected_camera_id(&self) -> Option<&str> {
        self.selected_camera_id.as_deref()
    }

    pub fn detected_camera(&self) -> Option<&Camera> {
        self.selected_camera_id
            .as_ref()
            .and_then(|id| self.available_cameras.iter().find(|c| &c.id == id))
    }

    pub fn current_settings(&self) -> Option<&CameraSettings> {
        self.current_settings.as_ref()
    }

    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    pub fn is_streaming(&self) -> bool {
        self.liveview_usecase.is_streaming()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Cinemaker Studio - LiveView");
        ui.separator();

        let mut refresh_requested = false;
        let mut toggle_liveview_requested = false;
        let previous_selected_id = self.selected_camera_id.clone();

        if self.liveview_usecase.is_streaming()
            && let Some(settings) = self.liveview_usecase.get_current_settings()
        {
            self.current_settings = Some(settings);
        }

        CameraPanel::show(
            ui,
            &self.available_cameras,
            &mut self.selected_camera_id,
            self.current_settings.as_ref(),
            self.liveview_usecase.is_streaming(),
            || {
                refresh_requested = true;
            },
            || {
                toggle_liveview_requested = true;
            },
        );

        // Si l'utilisateur a changé la caméra dans la ComboBox
        if self.selected_camera_id != previous_selected_id {
            self.refresh_settings();
            if self.liveview_usecase.is_streaming() {
                self.start_liveview();
            }
        }

        if refresh_requested {
            self.refresh_camera();
        }

        if toggle_liveview_requested {
            self.toggle_liveview();
        }

        ui.separator();

        LiveViewPanel::show(
            ui,
            &self.frame_buffer,
            &mut self.texture,
            self.current_settings.as_ref(),
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
