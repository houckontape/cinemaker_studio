use camera_domain::{Camera, CameraSettings};
use egui::{Color32, RichText, Ui};

pub struct CameraPanel<'a> {
    cameras: &'a [Camera],
    selected_camera_id: &'a mut Option<String>,
    settings: Option<&'a CameraSettings>,
    is_streaming: bool,
}

impl<'a> CameraPanel<'a> {
    pub fn new(
        cameras: &'a [Camera],
        selected_camera_id: &'a mut Option<String>,
        settings: Option<&'a CameraSettings>,
        is_streaming: bool,
    ) -> Self {
        Self {
            cameras,
            selected_camera_id,
            settings,
            is_streaming,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        cameras: &[Camera],
        selected_camera_id: &mut Option<String>,
        settings: Option<&CameraSettings>,
        is_streaming: bool,
        mut on_refresh: impl FnMut(),
        mut on_toggle_liveview: impl FnMut(),
        mut on_capture: impl FnMut(),
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Caméra :").strong());

            let current_name = if let Some(selected_id) = selected_camera_id.as_ref() {
                cameras
                    .iter()
                    .find(|c| &c.id == selected_id)
                    .map(|c| format!("{} ({:?})", c.name, c.kind))
                    .unwrap_or_else(|| "Sélectionnez une caméra".to_string())
            } else {
                "Sélectionnez une caméra".to_string()
            };

            egui::ComboBox::from_id_salt("camera_selector_combo")
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    if cameras.is_empty() {
                        ui.label("Aucune caméra trouvée");
                    } else {
                        for camera in cameras {
                            let label = format!("{} ({:?})", camera.name, camera.kind);
                            let is_selected = selected_camera_id.as_deref() == Some(&camera.id);
                            if ui
                                .selectable_label(is_selected, label)
                                .clicked()
                            {
                                *selected_camera_id = Some(camera.id.clone());
                            }
                        }
                    }
                });

            if ui.button("🔄 Rafraîchir").clicked() {
                on_refresh();
            }

            let liveview_btn_text = if is_streaming {
                RichText::new("⏹ Arrêter le LiveView").color(Color32::from_rgb(230, 80, 80))
            } else {
                RichText::new("▶ Lancer le LiveView").color(Color32::from_rgb(60, 180, 100))
            };

            let can_start_liveview = selected_camera_id.is_some() || is_streaming;
            if ui.add_enabled(can_start_liveview, egui::Button::new(liveview_btn_text)).clicked() {
                on_toggle_liveview();
            }

            let capture_btn = egui::Button::new(
                RichText::new("📸 Prendre une photo")
                    .color(Color32::from_rgb(255, 255, 255))
                    .strong(),
            )
            .fill(Color32::from_rgb(40, 110, 200));

            if ui.add_enabled(is_streaming, capture_btn).clicked() {
                on_capture();
            }
        });

        if let Some(s) = settings
            && !s.is_empty()
        {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Réglages :").strong());

                if !s.shooting_mode.is_empty() {
                    ui.label(format!("📷 Mode: {}", s.shooting_mode));
                    ui.separator();
                }
                if !s.iso.is_empty() {
                    ui.label(format!("ISO: {}", s.iso));
                    ui.separator();
                }
                if !s.aperture.is_empty() {
                    ui.label(format!("Ouverture: {}", s.aperture));
                    ui.separator();
                }
                if !s.shutter_speed.is_empty() {
                    ui.label(format!("Vitesse: {}", s.shutter_speed));
                    ui.separator();
                }
                if !s.exposure_comp.is_empty() {
                    ui.label(format!("Exp: {}", s.exposure_comp));
                    ui.separator();
                }
                if !s.white_balance.is_empty() {
                    ui.label(format!("Bal. Blancs: {}", s.white_balance));
                    ui.separator();
                }
                if !s.battery_level.is_empty() {
                    ui.label(format!("🔋 {}", s.battery_level));
                }
            });
        }
    }

    pub fn ui(
        self,
        ui: &mut Ui,
        on_refresh: impl FnMut(),
        on_toggle_liveview: impl FnMut(),
        on_capture: impl FnMut(),
    ) {
        Self::show(
            ui,
            self.cameras,
            self.selected_camera_id,
            self.settings,
            self.is_streaming,
            on_refresh,
            on_toggle_liveview,
            on_capture,
        );
    }
}
