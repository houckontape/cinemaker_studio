use camera_domain::{CameraSettings, FrameBuffer};
use egui::{load::SizedTexture, Color32, ColorImage, Frame, RichText, TextureHandle, TextureOptions, Ui, Vec2};

pub struct LiveViewPanel;

impl LiveViewPanel {
    pub fn show(
        ui: &mut Ui,
        buffer: &FrameBuffer,
        texture: &mut Option<TextureHandle>,
        settings: Option<&CameraSettings>,
        is_streaming: bool,
    ) {
        Frame::canvas(ui.style())
            .fill(Color32::from_rgb(20, 20, 24))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if !is_streaming {
                        ui.add_space(80.0);
                        ui.label(
                            RichText::new("📷 LiveView inactif")
                                .size(18.0)
                                .color(Color32::from_rgb(180, 180, 190)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Sélectionnez votre caméra et cliquez sur 'Lancer le LiveView'")
                                .size(13.0)
                                .color(Color32::from_rgb(120, 120, 130)),
                        );
                        ui.add_space(80.0);
                        return;
                    }

                    let frame_opt = if let Ok(lock) = buffer.lock() {
                        lock.clone()
                    } else {
                        None
                    };

                    if let Some(frame) = frame_opt {
                        if frame.width > 0 && frame.height > 0 {
                            let expected_len = (frame.width * frame.height * 4) as usize;
                            let rgba_data = if frame.data.len() == expected_len {
                                frame.data
                            } else if frame.data.len() == (frame.width * frame.height * 3) as usize {
                                let mut rgba = Vec::with_capacity(expected_len);
                                let (chunks, _) = frame.data.as_chunks::<3>();
                                for chunk in chunks {
                                    rgba.push(chunk[0]);
                                    rgba.push(chunk[1]);
                                    rgba.push(chunk[2]);
                                    rgba.push(255);
                                }
                                rgba
                            } else {
                                vec![0u8; expected_len]
                            };

                            let color_image = ColorImage::from_rgba_unmultiplied(
                                [frame.width as usize, frame.height as usize],
                                &rgba_data,
                            );

                            let tex = texture.get_or_insert_with(|| {
                                ui.ctx().load_texture(
                                    "liveview_stream",
                                    color_image.clone(),
                                    TextureOptions::LINEAR,
                                )
                            });
                            tex.set(color_image, TextureOptions::LINEAR);

                            let available_size = ui.available_size();
                            let aspect_ratio = frame.width as f32 / frame.height as f32;

                            // Déduire l'espace nécessaire pour la barre d'état en dessous du flux
                            let status_bar_height = 28.0;
                            let max_height = (available_size.y - status_bar_height).max(0.0);

                            let mut display_width = available_size.x;
                            let mut display_height = display_width / aspect_ratio;

                            if display_height > max_height && max_height > 0.0 {
                                display_height = max_height;
                                display_width = display_height * aspect_ratio;
                            }

                            if display_width <= 0.0 || display_height <= 0.0 {
                                display_width = 640.0;
                                display_height = 640.0 / aspect_ratio;
                            }

                            let sized_texture =
                                SizedTexture::new(tex.id(), Vec2::new(display_width, display_height));
                            ui.image(sized_texture);

                            // Status bar under liveview
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("● LIVE ({}x{})", frame.width, frame.height))
                                        .size(11.0)
                                        .color(Color32::from_rgb(80, 220, 100)),
                                );
                                if let Some(s) = settings
                                    && !s.is_empty()
                                {
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!(
                                            "Mode: {} | ISO: {} | {} | {} | Exp: {} | WB: {}",
                                            s.shooting_mode,
                                            s.iso,
                                            s.aperture,
                                            s.shutter_speed,
                                            s.exposure_comp,
                                            s.white_balance
                                        ))
                                        .size(11.0)
                                        .color(Color32::from_rgb(200, 200, 210)),
                                    );
                                }
                            });
                        } else {
                            ui.add_space(80.0);
                            ui.spinner();
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Initialisation du capteur et du flux vidéo...")
                                    .color(Color32::from_rgb(200, 200, 200)),
                            );
                            ui.add_space(80.0);
                        }
                    } else {
                        ui.add_space(80.0);
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Connexion au flux LiveView...")
                                .color(Color32::from_rgb(200, 200, 200)),
                        );
                        ui.add_space(80.0);
                    }
                });
            });
    }
}
