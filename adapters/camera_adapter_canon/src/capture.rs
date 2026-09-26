use crate::canon_adapter::CanonSdk;
use crate::edsdk_sys::*;
use crate::error::{CanonError, CanonResult};

pub struct CaptureController;

impl CaptureController {
    pub fn take_picture(camera_index: usize) -> CanonResult<()> {
        CanonSdk::init()?;
        unsafe {
            let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
            let err = EdsGetCameraList(&mut camera_list);
            if err != EDS_ERR_OK || camera_list.is_null() {
                return Err(CanonError::EdsError(err));
            }

            let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
            let err = EdsGetChildAtIndex(camera_list, camera_index as EdsInt32, &mut cam_ref);
            if err != EDS_ERR_OK || cam_ref.is_null() {
                EdsRelease(camera_list);
                return Err(CanonError::InvalidCameraIndex(camera_index));
            }

            let open_err = EdsOpenSession(cam_ref);
            if open_err != EDS_ERR_OK && open_err != EDS_ERR_SESSION_ALREADY_OPEN {
                EdsRelease(cam_ref);
                EdsRelease(camera_list);
                return Err(CanonError::SessionError(open_err));
            }

            let err = EdsSendCommand(cam_ref, kEdsCameraCommand_TakePicture, 0);

            if open_err == EDS_ERR_OK {
                EdsCloseSession(cam_ref);
            }
            EdsRelease(cam_ref);
            EdsRelease(camera_list);

            if err == EDS_ERR_OK {
                Ok(())
            } else {
                Err(CanonError::EdsError(err))
            }
        }
    }
}
