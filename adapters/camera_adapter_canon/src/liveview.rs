use crate::edsdk_sys::*;
use crate::error::{check_edsdk, CanonError, Result};
use crate::canon_adapter::CanonCameraAdapter;
use std::ptr;

pub struct LiveViewStream<'a> {
    adapter: &'a CanonCameraAdapter,
}

impl<'a> LiveViewStream<'a> {
    pub fn new(adapter: &'a CanonCameraAdapter) -> Result<Self> {
        let cam = adapter.raw_camera_ref()?;

        unsafe {
            let mut device: EdsUInt32 = 0;
            check_edsdk(EdsGetPropertyData(
                cam as *mut _,
                K_EDS_PROP_ID_EVF_OUTPUT_DEVICE,
                0,
                std::mem::size_of::<EdsUInt32>() as u32,
                &mut device as *mut _ as *mut _,
            ))?;

            device |= K_EDS_EVF_OUTPUT_DEVICE_PC;

            check_edsdk(EdsSetPropertyData(
                cam as *mut _,
                K_EDS_PROP_ID_EVF_OUTPUT_DEVICE,
                0,
                std::mem::size_of::<EdsUInt32>() as u32,
                &device as *const _ as *const _,
            ))?;
        }

        Ok(Self { adapter })
    }

    /// Télécharge une frame LiveView brute (buffer EVF).
    pub fn get_frame(&self) -> Result<Vec<u8>> {
        unsafe {
            let cam = self.adapter.raw_camera_ref()?;

            // Ici, dans un vrai code, tu créerais un stream (EdsCreateMemoryStream)
            // puis un EvfImageRef, puis EdsDownloadEvfImage.
            // On simplifie en supposant que K_EDS_PROP_ID_EVF_DATA renvoie un buffer.

            let mut buf = vec![0u8; 1024 * 1024]; // 1 Mo, à ajuster
            check_edsdk(EdsGetPropertyData(
                cam as *mut _,
                K_EDS_PROP_ID_EVF_DATA,
                0,
                buf.len() as u32,
                buf.as_mut_ptr() as *mut _,
            ))?;

            Ok(buf)
        }
    }
}
