use camera_domain::{Frame, FrameBuffer, Photo};
use camera_ports::PhotoRepository;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static PHOTO_COUNTER: AtomicU64 = AtomicU64::new(1);

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generate_photo_id() -> String {
    let count = PHOTO_COUNTER.fetch_add(1, Ordering::SeqCst);
    let ts = now_millis();
    format!("frame_{:04}_{}", count, ts)
}

/// Cas d'utilisation : Capturer une photo et l'enregistrer dans la timeline avec sa miniature indissociable
pub struct CapturePhotoUseCase<R: PhotoRepository> {
    repository: R,
}

impl<R: PhotoRepository> CapturePhotoUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Capture une photo à partir d'une frame (ex: frame pleine résolution ou frame du buffer)
    pub fn capture_from_frame(&mut self, frame: Frame) -> Photo {
        let id = generate_photo_id();
        let timestamp = now_millis();
        let photo = Photo::create_default(id, frame, timestamp);
        self.repository.add_photo(photo.clone());
        photo
    }

    /// Capture une photo à partir du FrameBuffer du LiveView s'il contient une image
    pub fn capture_from_buffer(&mut self, buffer: &FrameBuffer) -> Option<Photo> {
        let frame_opt = if let Ok(lock) = buffer.lock() {
            lock.clone()
        } else {
            None
        };

        frame_opt.map(|frame| self.capture_from_frame(frame))
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn repository_mut(&mut self) -> &mut R {
        &mut self.repository
    }
}

/// Cas d'utilisation : Supprimer une photo de la timeline
pub struct DeletePhotoUseCase<R: PhotoRepository> {
    repository: R,
}

impl<R: PhotoRepository> DeletePhotoUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn delete_by_id(&mut self, id: &str) -> Option<Photo> {
        self.repository.remove_photo(id)
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn repository_mut(&mut self) -> &mut R {
        &mut self.repository
    }
}

/// Cas d'utilisation : Récupérer la liste des photos de la timeline
pub struct ListPhotosUseCase<R: PhotoRepository> {
    repository: R,
}

impl<R: PhotoRepository> ListPhotosUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Vec<Photo> {
        self.repository.get_photos()
    }

    pub fn count(&self) -> usize {
        self.repository.count()
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }
}

/// Service d'orchestration pour la timeline de Stop Motion
pub struct TimelineUseCase<R: PhotoRepository> {
    repository: R,
}

impl<R: PhotoRepository> TimelineUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn capture_from_frame(&mut self, frame: Frame) -> Photo {
        let id = generate_photo_id();
        let timestamp = now_millis();
        let photo = Photo::create_default(id, frame, timestamp);
        self.repository.add_photo(photo.clone());
        photo
    }

    pub fn capture_from_buffer(&mut self, buffer: &FrameBuffer) -> Option<Photo> {
        let frame_opt = if let Ok(lock) = buffer.lock() {
            lock.clone()
        } else {
            None
        };

        frame_opt.map(|frame| self.capture_from_frame(frame))
    }

    pub fn delete_photo(&mut self, id: &str) -> Option<Photo> {
        self.repository.remove_photo(id)
    }

    pub fn get_photos(&self) -> Vec<Photo> {
        self.repository.get_photos()
    }

    pub fn get_photo(&self, id: &str) -> Option<Photo> {
        self.repository.get_photo(id)
    }

    pub fn count(&self) -> usize {
        self.repository.count()
    }

    pub fn clear(&mut self) {
        self.repository.clear();
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn repository_mut(&mut self) -> &mut R {
        &mut self.repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_ports::InMemoryPhotoRepository;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_capture_and_delete_photo_usecases() {
        let repo = InMemoryPhotoRepository::new();
        let mut capture_uc = CapturePhotoUseCase::new(repo.clone());
        let mut delete_uc = DeletePhotoUseCase::new(repo.clone());
        let list_uc = ListPhotosUseCase::new(repo.clone());

        assert_eq!(list_uc.count(), 0);

        let frame = Frame {
            data: vec![128; 640 * 480 * 4],
            width: 640,
            height: 480,
        };

        let captured = capture_uc.capture_from_frame(frame);
        assert_eq!(list_uc.count(), 1);
        assert_eq!(captured.full_image.width, 640);
        assert_eq!(captured.thumbnail.width, 160);

        let deleted = delete_uc.delete_by_id(&captured.id);
        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().id, captured.id);
        assert_eq!(list_uc.count(), 0);
    }

    #[test]
    fn test_timeline_usecase_buffer_capture() {
        let repo = InMemoryPhotoRepository::new();
        let mut timeline_uc = TimelineUseCase::new(repo);

        let buffer: FrameBuffer = Arc::new(Mutex::new(None));
        assert!(timeline_uc.capture_from_buffer(&buffer).is_none());

        *buffer.lock().unwrap() = Some(Frame {
            data: vec![255; 800 * 600 * 4],
            width: 800,
            height: 600,
        });

        let photo = timeline_uc.capture_from_buffer(&buffer);
        assert!(photo.is_some());
        let p = photo.unwrap();
        assert_eq!(p.full_image.width, 800);
        assert_eq!(timeline_uc.count(), 1);

        // Delete photo
        let deleted = timeline_uc.delete_photo(&p.id);
        assert!(deleted.is_some());
        assert_eq!(timeline_uc.count(), 0);
    }
}
