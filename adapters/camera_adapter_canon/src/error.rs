use thiserror::Error;
use crate::edsdk_sys::EdsError;

#[derive(Error, Debug)]
pub enum CanonError {
    #[error("Erreur d'initialisation du SDK Canon (code: {0:#x})")]
    SdkInitFailed(EdsError),

    #[error("Aucune caméra Canon détectée")]
    CameraNotFound,

    #[error("Index de caméra invalide: {0}")]
    InvalidCameraIndex(usize),

    #[error("Erreur lors de l'ouverture de la session Canon (code: {0:#x})")]
    SessionError(EdsError),

    #[error("Erreur d'accès à la propriété Canon (code: {0:#x})")]
    PropertyError(EdsError),

    #[error("Erreur LiveView Canon (code: {0:#x})")]
    LiveViewError(EdsError),

    #[error("Erreur de décodage de l'image JPEG LiveView: {0}")]
    ImageDecodeError(String),

    #[error("Erreur EDSDK générique (code: {0:#x})")]
    EdsError(EdsError),

    #[error("Caméra Canon non connectée ou session fermée")]
    NotConnected,
}

pub type CanonResult<T> = Result<T, CanonError>;
