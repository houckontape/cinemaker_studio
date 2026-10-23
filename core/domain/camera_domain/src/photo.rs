use crate::Frame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Photo {
    pub id: String,
    pub full_image: Frame,
    pub thumbnail: Frame,
    pub created_at_timestamp: u64,
}

impl Photo {
    pub const DEFAULT_THUMBNAIL_WIDTH: u32 = 160;
    pub const DEFAULT_THUMBNAIL_HEIGHT: u32 = 120;

    /// Crée une photo en liant de manière indissociable l'image réelle et sa miniature
    pub fn new(
        id: impl Into<String>,
        full_image: Frame,
        thumbnail: Frame,
        created_at_timestamp: u64,
    ) -> Self {
        Self {
            id: id.into(),
            full_image,
            thumbnail,
            created_at_timestamp,
        }
    }

    /// Crée une photo et génère automatiquement sa miniature à partir de l'image réelle
    pub fn from_frame(
        id: impl Into<String>,
        full_image: Frame,
        max_thumb_w: u32,
        max_thumb_h: u32,
        created_at_timestamp: u64,
    ) -> Self {
        let thumbnail = Self::generate_thumbnail(&full_image, max_thumb_w, max_thumb_h);
        Self {
            id: id.into(),
            full_image,
            thumbnail,
            created_at_timestamp,
        }
    }

    /// Crée une photo avec la taille de miniature par défaut (160x120 max)
    pub fn create_default(
        id: impl Into<String>,
        full_image: Frame,
        created_at_timestamp: u64,
    ) -> Self {
        Self::from_frame(
            id,
            full_image,
            Self::DEFAULT_THUMBNAIL_WIDTH,
            Self::DEFAULT_THUMBNAIL_HEIGHT,
            created_at_timestamp,
        )
    }

    /// Génère une miniature avec redimensionnement bilinéaire préservant le ratio d'aspect
    pub fn generate_thumbnail(frame: &Frame, max_w: u32, max_h: u32) -> Frame {
        if frame.width == 0 || frame.height == 0 || max_w == 0 || max_h == 0 {
            return Frame {
                data: Vec::new(),
                width: 0,
                height: 0,
            };
        }

        let ratio = (max_w as f32 / frame.width as f32)
            .min(max_h as f32 / frame.height as f32)
            .min(1.0);

        let dst_w = (frame.width as f32 * ratio).round().max(1.0) as u32;
        let dst_h = (frame.height as f32 * ratio).round().max(1.0) as u32;

        let src_rgba = Self::ensure_rgba(frame);
        let src_w = frame.width;
        let src_h = frame.height;

        let mut dst_data = Vec::with_capacity((dst_w * dst_h * 4) as usize);

        for y in 0..dst_h {
            let src_y = ((y as f32 + 0.5) * (src_h as f32 / dst_h as f32) - 0.5).max(0.0);
            let y0 = (src_y.floor() as u32).min(src_h - 1);
            let y1 = (y0 + 1).min(src_h - 1);
            let wy = src_y - y0 as f32;

            for x in 0..dst_w {
                let src_x = ((x as f32 + 0.5) * (src_w as f32 / dst_w as f32) - 0.5).max(0.0);
                let x0 = (src_x.floor() as u32).min(src_w - 1);
                let x1 = (x0 + 1).min(src_w - 1);
                let wx = src_x - x0 as f32;

                let p00_idx = ((y0 * src_w + x0) * 4) as usize;
                let p10_idx = ((y0 * src_w + x1) * 4) as usize;
                let p01_idx = ((y1 * src_w + x0) * 4) as usize;
                let p11_idx = ((y1 * src_w + x1) * 4) as usize;

                for c in 0..4 {
                    let c00 = src_rgba.get(p00_idx + c).copied().unwrap_or(0) as f32;
                    let c10 = src_rgba.get(p10_idx + c).copied().unwrap_or(0) as f32;
                    let c01 = src_rgba.get(p01_idx + c).copied().unwrap_or(0) as f32;
                    let c11 = src_rgba.get(p11_idx + c).copied().unwrap_or(0) as f32;

                    let top = c00 * (1.0 - wx) + c10 * wx;
                    let bot = c01 * (1.0 - wx) + c11 * wx;
                    let val = (top * (1.0 - wy) + bot * wy).round().clamp(0.0, 255.0) as u8;

                    dst_data.push(val);
                }
            }
        }

        Frame {
            data: dst_data,
            width: dst_w,
            height: dst_h,
        }
    }

    fn ensure_rgba(frame: &Frame) -> Vec<u8> {
        let expected_rgba = (frame.width * frame.height * 4) as usize;
        let expected_rgb = (frame.width * frame.height * 3) as usize;

        if frame.data.len() == expected_rgba {
            frame.data.clone()
        } else if frame.data.len() == expected_rgb {
            let mut rgba = Vec::with_capacity(expected_rgba);
            let (chunks, _) = frame.data.as_chunks::<3>();
            for chunk in chunks {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
            rgba
        } else {
            let mut res = vec![0u8; expected_rgba];
            let copy_len = res.len().min(frame.data.len());
            res[..copy_len].copy_from_slice(&frame.data[..copy_len]);
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photo_creation_and_indissociability() {
        let full = Frame {
            data: vec![200; 640 * 480 * 4],
            width: 640,
            height: 480,
        };

        let photo = Photo::create_default("photo_1", full.clone(), 123456789);

        assert_eq!(photo.id, "photo_1");
        assert_eq!(photo.full_image.width, 640);
        assert_eq!(photo.full_image.height, 480);
        assert_eq!(photo.created_at_timestamp, 123456789);

        // Verify thumbnail dimensions preserve 4:3 ratio
        assert_eq!(photo.thumbnail.width, 160);
        assert_eq!(photo.thumbnail.height, 120);
        assert_eq!(photo.thumbnail.data.len(), 160 * 120 * 4);
    }

    #[test]
    fn test_thumbnail_from_rgb() {
        let full = Frame {
            data: vec![100; 300 * 200 * 3],
            width: 300,
            height: 200,
        };

        let thumb = Photo::generate_thumbnail(&full, 150, 100);
        assert_eq!(thumb.width, 150);
        assert_eq!(thumb.height, 100);
        assert_eq!(thumb.data.len(), 150 * 100 * 4);
    }
}
