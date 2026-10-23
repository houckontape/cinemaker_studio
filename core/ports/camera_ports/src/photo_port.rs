use camera_domain::{Frame, Photo};
use std::sync::{Arc, Mutex};

pub trait PhotoRepository: Send + Sync {
    fn add_photo(&mut self, photo: Photo);
    fn get_photos(&self) -> Vec<Photo>;
    fn remove_photo(&mut self, id: &str) -> Option<Photo>;
    fn get_photo(&self, id: &str) -> Option<Photo>;
    fn clear(&mut self);
    fn count(&self) -> usize;
}

pub trait CameraCapture: Send + Sync {
    fn capture_frame(&self, camera_id: &str) -> Option<Frame>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryPhotoRepository {
    photos: Arc<Mutex<Vec<Photo>>>,
}

impl InMemoryPhotoRepository {
    pub fn new() -> Self {
        Self {
            photos: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl PhotoRepository for InMemoryPhotoRepository {
    fn add_photo(&mut self, photo: Photo) {
        if let Ok(mut lock) = self.photos.lock() {
            lock.push(photo);
        }
    }

    fn get_photos(&self) -> Vec<Photo> {
        if let Ok(lock) = self.photos.lock() {
            lock.clone()
        } else {
            Vec::new()
        }
    }

    fn remove_photo(&mut self, id: &str) -> Option<Photo> {
        if let Ok(mut lock) = self.photos.lock() {
            lock.iter()
                .position(|p| p.id == id)
                .map(|pos| lock.remove(pos))
        } else {
            None
        }
    }

    fn get_photo(&self, id: &str) -> Option<Photo> {
        if let Ok(lock) = self.photos.lock() {
            lock.iter().find(|p| p.id == id).cloned()
        } else {
            None
        }
    }

    fn clear(&mut self) {
        if let Ok(mut lock) = self.photos.lock() {
            lock.clear();
        }
    }

    fn count(&self) -> usize {
        if let Ok(lock) = self.photos.lock() {
            lock.len()
        } else {
            0
        }
    }
}

impl<T: PhotoRepository + ?Sized> PhotoRepository for Box<T> {
    fn add_photo(&mut self, photo: Photo) {
        (**self).add_photo(photo);
    }

    fn get_photos(&self) -> Vec<Photo> {
        (**self).get_photos()
    }

    fn remove_photo(&mut self, id: &str) -> Option<Photo> {
        (**self).remove_photo(id)
    }

    fn get_photo(&self, id: &str) -> Option<Photo> {
        (**self).get_photo(id)
    }

    fn clear(&mut self) {
        (**self).clear();
    }

    fn count(&self) -> usize {
        (**self).count()
    }
}

impl<T: CameraCapture + ?Sized> CameraCapture for &T {
    fn capture_frame(&self, camera_id: &str) -> Option<Frame> {
        (**self).capture_frame(camera_id)
    }
}

impl<T: CameraCapture + ?Sized> CameraCapture for Box<T> {
    fn capture_frame(&self, camera_id: &str) -> Option<Frame> {
        (**self).capture_frame(camera_id)
    }
}

impl<T: CameraCapture + ?Sized> CameraCapture for Arc<T> {
    fn capture_frame(&self, camera_id: &str) -> Option<Frame> {
        (**self).capture_frame(camera_id)
    }
}
