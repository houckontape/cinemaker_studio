use std::sync::Mutex;
pub mod liveview_adapter;
pub use liveview_adapter::*;
use windows::{
    core::PWSTR,
    Win32::Media::MediaFoundation::*,
    Win32::System::Com::*,
};

use camera_domain::{CameraInfo, Frame};
use camera_ports::{CameraDetector, CameraStream};

pub struct MediaFoundationAdapter {
    reader: Mutex<Option<IMFSourceReader>>,
}

impl MediaFoundationAdapter {
    pub fn new() -> Self {
        unsafe {
            MFStartup(MF_VERSION, MFSTARTUP_FULL).unwrap();
        }
        Self { reader: Mutex::new(None) }
    }
}

impl Default for MediaFoundationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraDetector for MediaFoundationAdapter {
    fn list_cameras(&self) -> Vec<CameraInfo> {
        unsafe {
            let mut attributes: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attributes, 1).unwrap();
            let attributes = attributes.unwrap();
            attributes
                .SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .unwrap();

            let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
            let mut count = 0;

            MFEnumDeviceSources(&attributes, &mut devices, &mut count).unwrap();

            let slice = std::slice::from_raw_parts(devices, count as usize);

            let mut cameras = Vec::new();
            for (i, dev_opt) in slice.iter().enumerate() {
                if let Some(dev) = dev_opt {
                    let mut name_ptr = PWSTR::null();
                    let mut len = 0;
                    dev.GetAllocatedString(&MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &mut name_ptr, &mut len).unwrap();
                    let name = String::from_utf16_lossy(std::slice::from_raw_parts(name_ptr.0, len as usize));
                    CoTaskMemFree(Some(name_ptr.0 as _));
                    cameras.push(CameraInfo {
                        id: i.to_string(),
                        name,
                    });
                }
            }

            CoTaskMemFree(Some(devices as _));

            cameras
        }
    }
}

impl CameraStream for MediaFoundationAdapter {
    fn start_stream(&self, camera_id: &str) {
        unsafe {
            let id: usize = camera_id.parse().unwrap();

            let mut attributes: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attributes, 1).unwrap();
            let attributes = attributes.unwrap();
            attributes
                .SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .unwrap();

            let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
            let mut count = 0;

            MFEnumDeviceSources(&attributes, &mut devices, &mut count).unwrap();

            let slice = std::slice::from_raw_parts(devices, count as usize);
            let device = slice[id].as_ref().unwrap();

            let source = device.ActivateObject::<IMFMediaSource>().unwrap();
            let reader = MFCreateSourceReaderFromMediaSource(&source, None).unwrap();

            // Stocker le reader dans self
            if let Ok(mut lock) = self.reader.lock() {
                *lock = Some(reader);
            }

            CoTaskMemFree(Some(devices as _));
        }
    }

    fn get_frame(&self) -> Option<Frame> {
        unsafe {
            let lock = self.reader.lock().ok()?;
            let reader = lock.as_ref()?;

            let mut stream_index = 0;
            let mut flags = 0;
            let mut sample: Option<IMFSample> = None;

            reader
                .ReadSample(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                    0,
                    Some(&mut stream_index),
                    Some(&mut flags),
                    None,
                    Some(&mut sample),
                )
                .unwrap();

            let sample = sample?;

            let buffer = sample.ConvertToContiguousBuffer().unwrap();

            let mut data_ptr: *mut u8 = std::ptr::null_mut();
            let mut max_len = 0;
            let mut cur_len = 0;

            buffer.Lock(&mut data_ptr, Some(&mut max_len), Some(&mut cur_len)).unwrap();

            let slice = std::slice::from_raw_parts(data_ptr, cur_len as usize);
            let data = slice.to_vec();

            buffer.Unlock().unwrap();

            Some(Frame {
                data,
                width: 640,
                height: 480,
            })
        }
    }

    fn stop_stream(&self) {
        if let Ok(mut lock) = self.reader.lock() {
            *lock = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_ports::CameraLiveView;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_liveview_hardware_camera() {
        let detector = MediaFoundationAdapter::new();
        let cameras = detector.list_cameras();
        println!("Available cameras: {:#?}", cameras);

        if let Some(first_camera) = cameras.first() {
            let liveview = MediaFoundationLiveViewAdapter::new();
            let buffer = Arc::new(Mutex::new(None));

            liveview.start_liveview(&first_camera.id, Arc::clone(&buffer));
            assert!(liveview.is_streaming());

            // Wait a bit for frames to arrive (up to 4 seconds for camera hardware to initialize)
            let mut received_frame = false;
            for _ in 0..40 {
                thread::sleep(Duration::from_millis(100));
                if let Ok(lock) = buffer.lock() {
                    if let Some(frame) = lock.as_ref() {
                        println!(
                            "Received hardware frame: {}x{}, data bytes: {}",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                        assert!(frame.width > 0);
                        assert!(frame.height > 0);
                        assert_eq!(frame.data.len(), (frame.width * frame.height * 4) as usize);
                        received_frame = true;
                        break;
                    }
                }
            }

            liveview.stop_liveview();
            assert!(!liveview.is_streaming());
            assert!(received_frame, "Should have received at least one frame from hardware camera");
        }
    }
}
