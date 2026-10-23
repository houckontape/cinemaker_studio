use camera_domain::Photo;
use egui::{
    load::SizedTexture, Color32, ColorImage, CornerRadius, Frame, Margin, RichText, Stroke,
    TextureHandle, TextureOptions, Ui, Vec2,
};
use std::collections::HashMap;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(
        ui: &mut Ui,
        photos: &[Photo],
        selected_photo_id: &mut Option<String>,
        thumbnail_textures: &mut HashMap<String, TextureHandle>,
        mut on_delete: impl FnMut(&str),
        mut on_clear: impl FnMut(),
    ) {
        Frame::canvas(ui.style())
            .fill(Color32::from_rgb(25, 25, 30))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("🎞️ Frise Chronologique ({} photos)", photos.len()))
                            .strong()
                            .size(14.0)
                            .color(Color32::from_rgb(220, 220, 230)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !photos.is_empty()
                            && ui
                                .button(
                                    RichText::new("🗑️ Tout effacer")
                                        .size(12.0)
                                        .color(Color32::from_rgb(220, 100, 100)),
                                )
                                .clicked()
                        {
                            on_clear();
                        }
                    });
                });

                ui.add_space(6.0);

                if photos.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Aucune photo dans la séquence de stop motion.")
                                .size(13.0)
                                .color(Color32::from_rgb(130, 130, 140)),
                        );
                        ui.label(
                            RichText::new("Cliquez sur '📸 Prendre une photo' pour ajouter la première image.")
                                .size(11.0)
                                .color(Color32::from_rgb(100, 100, 110)),
                        );
                        ui.add_space(20.0);
                    });
                    return;
                }

                egui::ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut delete_id: Option<String> = None;

                            for (index, photo) in photos.iter().enumerate() {
                                let is_selected = selected_photo_id.as_deref() == Some(&photo.id);

                                let border_color = if is_selected {
                                    Color32::from_rgb(80, 160, 255)
                                } else {
                                    Color32::from_rgb(50, 50, 60)
                                };

                                Frame::group(ui.style())
                                    .fill(if is_selected {
                                        Color32::from_rgb(35, 40, 50)
                                    } else {
                                        Color32::from_rgb(30, 30, 36)
                                    })
                                    .stroke(Stroke::new(if is_selected { 2.0 } else { 1.0 }, border_color))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(Margin::same(6))
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            // Numéro de l'image (Stop Motion Frame #)
                                            ui.label(
                                                RichText::new(format!("#{:03}", index + 1))
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Color32::from_rgb(180, 180, 190)),
                                            );

                                            ui.add_space(2.0);

                                            // Miniature
                                            let tex = thumbnail_textures
                                                .entry(photo.id.clone())
                                                .or_insert_with(|| {
                                                    let thumb = &photo.thumbnail;
                                                    let expected_len =
                                                        (thumb.width * thumb.height * 4) as usize;
                                                    let rgba_data = if thumb.data.len() == expected_len {
                                                        thumb.data.clone()
                                                    } else {
                                                        vec![0u8; expected_len]
                                                    };

                                                    let color_image =
                                                        ColorImage::from_rgba_unmultiplied(
                                                            [
                                                                thumb.width as usize,
                                                                thumb.height as usize,
                                                            ],
                                                            &rgba_data,
                                                        );

                                                    ui.ctx().load_texture(
                                                        format!("thumb_{}", photo.id),
                                                        color_image,
                                                        TextureOptions::LINEAR,
                                                    )
                                                });

                                            let thumb_w = 110.0;
                                            let thumb_h = if photo.thumbnail.width > 0 {
                                                (thumb_w * photo.thumbnail.height as f32)
                                                    / photo.thumbnail.width as f32
                                            } else {
                                                75.0
                                            };

                                            let img_response = ui.add(
                                                egui::Image::new(SizedTexture::new(
                                                    tex.id(),
                                                    Vec2::new(thumb_w, thumb_h),
                                                ))
                                                .sense(egui::Sense::click()),
                                            );

                                            if img_response.clicked() {
                                                *selected_photo_id = Some(photo.id.clone());
                                            }

                                            ui.add_space(4.0);

                                            // Bouton de suppression
                                            let del_btn = egui::Button::new(
                                                RichText::new("🗑️ Supprimer")
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(230, 90, 90)),
                                            );

                                            if ui.add(del_btn).clicked() {
                                                delete_id = Some(photo.id.clone());
                                            }
                                        });
                                    });

                                ui.add_space(4.0);
                            }

                            if let Some(id) = delete_id {
                                if selected_photo_id.as_deref() == Some(&id) {
                                    *selected_photo_id = None;
                                }
                                on_delete(&id);
                            }
                        });
                    });
            });
    }
}
