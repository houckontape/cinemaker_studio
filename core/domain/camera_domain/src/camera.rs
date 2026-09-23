#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraKind {
    Internal,
    Usb,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Camera {
    pub id: String,
    pub name: String,
    pub kind: CameraKind,
}

impl Camera {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: CameraKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
        }
    }
}
