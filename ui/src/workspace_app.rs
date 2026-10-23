use camera_domain::{Camera, CameraSettings, FrameBuffer, Photo};
use camera_ports::{CameraLiveView, CameraPort, InMemoryPhotoRepository, PhotoRepository};
use camera_usecases::{LiveViewUseCase, TimelineUseCase};
use crate::adapters::camera_adapter::{CameraAdapter, UnifiedLiveViewAdapter};
use crate::components::camera_panel::CameraPanel;
use crate::components::timeline_panel::TimelinePanel;
use crate::liveview_panel::LiveViewPanel;
use eframe::App;
use egui::{TextureHandle, Ui};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WorkspaceApp<
    P: CameraPort = CameraAdapter,
    L: CameraLiveView = UnifiedLiveViewAdapter,
    R: PhotoRepository = InMemoryPhotoRepository,
> {
    camera_port: P,
    liveview_usecase: LiveViewUseCase<L>,
    timeline_usecase: TimelineUseCase<R>,
    available_cameras: Vec<Camera>,
    selected_camera_id: Option<String>,
    current_settings: Option<CameraSettings>,
    frame_buffer: FrameBuffer,
    texture: Option<TextureHandle>,
    thumbnail_textures: HashMap<String, TextureHandle>,
    selected_photo_id: Option<String>,
}

impl Default for WorkspaceApp<CameraAdapter, UnifiedLiveViewAdapter, InMemoryPhotoRepository> {
    fn default() -> Self {
        let adapter = CameraAdapter::new();
        let liveview_adapter = UnifiedLiveViewAdapter::new();
        let repo = InMemoryPhotoRepository::new();
        Self::new_with_all_ports(adapter, liveview_adapter, repo)
    }
}

impl<P: CameraPort, L: CameraLiveView> WorkspaceApp<P, L, InMemoryPhotoRepository> {
    pub fn new_with_ports(camera_port: P, liveview_port: L) -> Self {
        let repo = InMemoryPhotoRepository::new();
        Self::new_with_all_ports(camera_port, liveview_port, repo)
    }

    pub fn new(camera_port: P) -> WorkspaceApp<P, UnifiedLiveViewAdapter, InMemoryPhotoRepository> {
        let liveview_adapter = UnifiedLiveViewAdapter::new();
        let repo = InMemoryPhotoRepository::new();
        WorkspaceApp::new_with_all_ports(camera_port, liveview_adapter, repo)
    }
}

impl<P: CameraPort, L: CameraLiveView, R: PhotoRepository> WorkspaceApp<P, L, R> {
    pub fn new_with_all_ports(camera_port: P, liveview_port: L, photo_repo: R) -> Self {
        let available_cameras = camera_port.list_cameras();
        let selected_camera_id = available_cameras.first().map(|c| c.id.clone());
        let current_settings = selected_camera_id
            .as_deref()
            .and_then(|id| camera_port.get_camera_settings(id));
        let liveview_usecase = LiveViewUseCase::new(liveview_port);
        let timeline_usecase = TimelineUseCase::new(photo_repo);
        let frame_buffer: FrameBuffer = Arc::new(Mutex::new(None));

        Self {
            camera_port,
            liveview_usecase,
            timeline_usecase,
            available_cameras,
            selected_camera_id,
            current_settings,
            frame_buffer,
            texture: None,
            thumbnail_textures: HashMap::new(),
            selected_photo_id: None,
        }
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
        if self.liveview_usecase.is_streaming() {
            if let Some(settings) = self.liveview_usecase.get_current_settings() {
                self.current_settings = Some(settings);
            }
            return;
        }

        if let Some(ref selected_id) = self.selected_camera_id {
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

    pub fn capture_photo(&mut self) -> Option<Photo> {
        self.timeline_usecase.capture_from_buffer(&self.frame_buffer)
    }

    pub fn delete_photo(&mut self, id: &str) -> Option<Photo> {
        self.thumbnail_textures.remove(id);
        if self.selected_photo_id.as_deref() == Some(id) {
            self.selected_photo_id = None;
        }
        self.timeline_usecase.delete_photo(id)
    }

    pub fn clear_timeline(&mut self) {
        self.thumbnail_textures.clear();
        self.selected_photo_id = None;
        self.timeline_usecase.clear();
    }

    pub fn timeline_photos(&self) -> Vec<Photo> {
        self.timeline_usecase.get_photos()
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

    pub fn selected_photo_id(&self) -> Option<&str> {
        self.selected_photo_id.as_deref()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Cinemaker Studio - Prise de vue Stop Motion");
        ui.separator();

        let mut refresh_requested = false;
        let mut toggle_liveview_requested = false;
        let mut capture_requested = false;
        let previous_selected_id = self.selected_camera_id.clone();

        // Raccourci clavier pour capturer : Espace ou Entrée
        if self.liveview_usecase.is_streaming()
            && ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter))
        {
            capture_requested = true;
        }

        if self.liveview_usecase.is_streaming()
            && let Some(settings) = self.liveview_usecase.get_current_settings()
        {
            self.current_settings = Some(settings);
        }

        // Panneau supérieur : En-tête et contrôle de la caméra
        egui::Panel::top("top_controls_panel").show(ui, |ui| {
            

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
                || {
                    capture_requested = true;
                },
            );
        });

        // Frise chronologique en bas de l'écran (toujours visible, même avec le LiveView actif)
        let photos = self.timeline_usecase.get_photos();
        let mut photo_to_delete: Option<String> = None;
        let mut clear_all_requested = false;

        egui::Panel::bottom("bottom_timeline_panel")
            .show(ui, |ui| {
                TimelinePanel::show(
                    ui,
                    &photos,
                    &mut self.selected_photo_id,
                    &mut self.thumbnail_textures,
                    |id| {
                        photo_to_delete = Some(id.to_string());
                    },
                    || {
                        clear_all_requested = true;
                    },
                );
            });

        // Zone centrale : Flux LiveView qui s'adapte à l'espace restant entre le haut et la frise
        egui::CentralPanel::default().show(ui, |ui| {
            LiveViewPanel::show(
                ui,
                &self.frame_buffer,
                &mut self.texture,
                self.current_settings.as_ref(),
                self.liveview_usecase.is_streaming(),
            );
        });

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

        if capture_requested {
            self.capture_photo();
        }

        if let Some(id) = photo_to_delete {
            self.delete_photo(&id);
        }

        if clear_all_requested {
            self.clear_timeline();
        }

        if self.liveview_usecase.is_streaming() {
            ui.ctx().request_repaint();
        }
    }
}

impl<P: CameraPort, L: CameraLiveView, R: PhotoRepository> App for WorkspaceApp<P, L, R> {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.ui(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_domain::{CameraKind, Frame};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockCameraPort {
        cams: Vec<Camera>,
    }
    impl CameraPort for MockCameraPort {
        fn list_cameras(&self) -> Vec<Camera> {
            self.cams.clone()
        }
    }

    struct MockLiveView {
        streaming: Arc<AtomicBool>,
    }
    impl CameraLiveView for MockLiveView {
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
    fn test_workspace_app_photo_capture_and_timeline() {
        let mock_port = MockCameraPort {
            cams: vec![Camera::new("cam_1", "Test Cam", CameraKind::Usb)],
        };
        let streaming_flag = Arc::new(AtomicBool::new(false));
        let mock_lv = MockLiveView {
            streaming: Arc::clone(&streaming_flag),
        };

        let mut app = WorkspaceApp::new_with_ports(mock_port, mock_lv);
        assert_eq!(app.timeline_photos().len(), 0);

        // Before liveview start, buffer is empty
        let photo = app.capture_photo();
        assert!(photo.is_none());

        // Start liveview
        app.start_liveview();
        assert!(app.is_streaming());

        // Capture photo from liveview
        let captured = app.capture_photo();
        assert!(captured.is_some());
        let p = captured.unwrap();
        assert_eq!(p.full_image.width, 640);
        assert_eq!(p.thumbnail.width, 160);
        assert_eq!(app.timeline_photos().len(), 1);
        assert_eq!(app.timeline_photos()[0].id, p.id);

        // Capture a second photo
        let captured2 = app.capture_photo().unwrap();
        assert_eq!(app.timeline_photos().len(), 2);

        // Delete first photo
        let deleted = app.delete_photo(&p.id);
        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().id, p.id);
        assert_eq!(app.timeline_photos().len(), 1);
        assert_eq!(app.timeline_photos()[0].id, captured2.id);

        // Clear all
        app.clear_timeline();
        assert_eq!(app.timeline_photos().len(), 0);
    }

    #[test]
    fn test_workspace_app_ui_rendering_with_liveview_and_timeline() {
        let mock_port = MockCameraPort {
            cams: vec![Camera::new("cam_1", "Test Cam", CameraKind::Usb)],
        };
        let streaming_flag = Arc::new(AtomicBool::new(false));
        let mock_lv = MockLiveView {
            streaming: Arc::clone(&streaming_flag),
        };

        let mut app = WorkspaceApp::new_with_ports(mock_port, mock_lv);
        app.start_liveview();
        let _ = app.capture_photo();

        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.ui(ui);
            });
        });
        output.textures_delta.clear();

        assert_eq!(app.timeline_photos().len(), 1);
        assert!(app.is_streaming());
    }
}
