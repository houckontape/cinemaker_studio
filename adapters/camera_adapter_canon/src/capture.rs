use crate::edsdk_sys::*;
use crate::error::{check_edsdk, Result};
use crate::canon_adapter::CanonCameraAdapter;

pub struct CaptureController<'a> {
    adapter: &'a CanonCameraAdapter,
}

impl<'a> CaptureController<'a> {
    pub fn new(adapter: &'a CanonCameraAdapter) -> Self {
        Self { adapter }
    }

    pub fn capture_still(&self) -> Result<()> {
        unsafe {
            let cam = self.adapter.raw_camera_ref()?;

            check_edsdk(EdsSendCommand(
                cam,
                K_EDS_CAMERA_COMMAND_PRESS_SHUTTER_BUTTON,
                K_EDS_CAMERA_COMMAND_SHUTTER_BUTTON_COMPLETELY,
            ))?;

            check_edsdk(EdsSendCommand(
                cam,
                K_EDS_CAMERA_COMMAND_PRESS_SHUTTER_BUTTON,
                K_EDS_CAMERA_COMMAND_SHUTTER_BUTTON_OFF,
            ))?;
        }

        Ok(())
    }
}
