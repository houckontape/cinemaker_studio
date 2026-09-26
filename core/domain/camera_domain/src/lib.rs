pub mod camera;
pub mod camera_settings;
pub mod liveview_domain;
pub use camera::*;
pub use camera_settings::*;
pub use liveview_domain::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}