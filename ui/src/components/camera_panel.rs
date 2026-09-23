use camera_domain::Camera;
use egui::Ui;

pub struct CameraPanel<'a> {
    detected_camera: &'a Option<Camera>,
}

impl<'a> CameraPanel<'a> {
    pub fn new(detected_camera: &'a Option<Camera>) -> Self {
        Self { detected_camera }
    }

    pub fn show(ui: &mut Ui, detected_camera: &Option<Camera>, mut on_refresh: impl FnMut()) {
        ui.horizontal(|ui| {
            match detected_camera {
                Some(camera) => {
                    ui.label(format!("Camera détectée : {}", camera.name));
                }
                None => {
                    ui.label("Aucune caméra détectée");
                }
            }

            if ui.button("Rafraîchir").clicked() {
                on_refresh();
            }
        });
    }

    pub fn ui(self, ui: &mut Ui, on_refresh: impl FnMut()) {
        Self::show(ui, self.detected_camera, on_refresh);
    }
}
