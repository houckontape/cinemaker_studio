use camera_domain::FrameBuffer;
use egui::{load::SizedTexture, ColorImage, TextureHandle, TextureOptions, Ui, Vec2};

pub struct LiveViewPanel;

impl LiveViewPanel {
    pub fn show(
        ui: &mut Ui,
        buffer: &FrameBuffer,
        texture: &mut Option<TextureHandle>,
        is_streaming: bool,
    ) {
        ui.vertical_centered(|ui| {
            if !is_streaming {
                ui.label("LiveView inactif");
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

                    let mut display_width = available_size.x;
                    let mut display_height = display_width / aspect_ratio;

                    if display_height > available_size.y && available_size.y > 0.0 {
                        display_height = available_size.y;
                        display_width = display_height * aspect_ratio;
                    }

                    if display_width <= 0.0 || display_height <= 0.0 {
                        display_width = 640.0;
                        display_height = 640.0 / aspect_ratio;
                    }

                    let sized_texture = SizedTexture::new(tex.id(), Vec2::new(display_width, display_height));
                    ui.image(sized_texture);
                } else {
                    ui.label("En attente du flux vidéo...");
                }
            } else {
                ui.label("En attente du flux vidéo...");
            }
        });
    }
}
