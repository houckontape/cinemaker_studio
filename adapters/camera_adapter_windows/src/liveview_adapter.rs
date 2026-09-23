use camera_domain::{Frame, FrameBuffer};
use camera_ports::CameraLiveView;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use windows::{
    Win32::Media::MediaFoundation::*,
    Win32::System::Com::*,
};

pub struct MediaFoundationLiveViewAdapter {
    is_running: Arc<AtomicBool>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
}

impl MediaFoundationLiveViewAdapter {
    pub fn new() -> Self {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            thread_handle: Mutex::new(None),
        }
    }
}

impl Default for MediaFoundationLiveViewAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MediaFoundationLiveViewAdapter {
    fn drop(&mut self) {
        self.stop_liveview();
    }
}

impl CameraLiveView for MediaFoundationLiveViewAdapter {
    fn start_liveview(&self, camera_id: &str, buffer: FrameBuffer) {
        self.stop_liveview();

        let camera_id_string = camera_id.to_string();
        let running = Arc::clone(&self.is_running);
        running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            let id: usize = match camera_id_string.parse() {
                Ok(id) => id,
                Err(_) => {
                    running.store(false, Ordering::SeqCst);
                    return;
                }
            };

            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            }

            let capture_result = (|| -> windows::core::Result<()> {
                let (reader, mut frame_width, mut frame_height) = unsafe {
                    let mut attributes: Option<IMFAttributes> = None;
                    MFCreateAttributes(&mut attributes, 1)?;
                    let attributes = attributes.ok_or_else(windows::core::Error::empty)?;

                    attributes.SetGUID(
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                    )?;

                    let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
                    let mut count = 0;

                    MFEnumDeviceSources(&attributes, &mut devices, &mut count)?;

                    if count == 0 || devices.is_null() {
                        if !devices.is_null() {
                            CoTaskMemFree(Some(devices as _));
                        }
                        return Err(windows::core::Error::empty());
                    }

                    let slice = std::slice::from_raw_parts(devices, count as usize);
                    if id >= slice.len() || slice[id].is_none() {
                        CoTaskMemFree(Some(devices as _));
                        return Err(windows::core::Error::empty());
                    }

                    let device = slice[id].as_ref().unwrap();
                    let source = device.ActivateObject::<IMFMediaSource>()?;
                    CoTaskMemFree(Some(devices as _));

                    // Configure SourceReader attributes to enable color conversion
                    let mut reader_attributes: Option<IMFAttributes> = None;
                    let _ = MFCreateAttributes(&mut reader_attributes, 1);
                    if let Some(ref rattr) = reader_attributes {
                        let _ = rattr.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1);
                    }

                    let reader = MFCreateSourceReaderFromMediaSource(&source, reader_attributes.as_ref())?;

                    // Request uncompressed RGB32
                    if let Ok(media_type) = MFCreateMediaType() {
                        let _ = media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video);
                        let _ = media_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32);
                        let _ = reader.SetCurrentMediaType(
                            MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                            None,
                            &media_type,
                        );
                    }

                    // Retrieve actual frame resolution
                    let mut width = 640u32;
                    let mut height = 480u32;
                    if let Ok(current_type) = reader.GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                        && let Ok(size_u64) = current_type.GetUINT64(&MF_MT_FRAME_SIZE)
                    {
                        width = (size_u64 >> 32) as u32;
                        height = (size_u64 & 0xFFFF_FFFF) as u32;
                    }

                    (reader, width, height)
                };

                // Target ~30 FPS (around 33ms per frame)
                let frame_duration = Duration::from_millis(33);

                while running.load(Ordering::SeqCst) {
                    let start_time = Instant::now();

                    let frame_opt = unsafe {
                        let mut stream_index = 0;
                        let mut flags = 0;
                        let mut sample: Option<IMFSample> = None;

                        let read_res = reader.ReadSample(
                            MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                            0,
                            Some(&mut stream_index),
                            Some(&mut flags),
                            None,
                            Some(&mut sample),
                        );

                        if read_res.is_ok() {
                            if let Some(sample) = sample {
                                if let Ok(buf) = sample.ConvertToContiguousBuffer() {
                                    let mut data_ptr: *mut u8 = std::ptr::null_mut();
                                    let mut max_len = 0;
                                    let mut cur_len = 0;

                                    if buf
                                        .Lock(&mut data_ptr, Some(&mut max_len), Some(&mut cur_len))
                                        .is_ok()
                                    {
                                        let slice =
                                            std::slice::from_raw_parts(data_ptr, cur_len as usize);
                                        
                                        // Update width/height if media format changed dynamically
                                        if let Ok(current_type) = reader.GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                                            && let Ok(size_u64) = current_type.GetUINT64(&MF_MT_FRAME_SIZE)
                                        {
                                            frame_width = (size_u64 >> 32) as u32;
                                            frame_height = (size_u64 & 0xFFFF_FFFF) as u32;
                                        }

                                        let expected_bgra_len = (frame_width * frame_height * 4) as usize;
                                        let rgba_data = if slice.len() >= expected_bgra_len {
                                            // Convert BGRA to RGBA
                                            let mut rgba = Vec::with_capacity(expected_bgra_len);
                                            let (chunks, _) = slice[..expected_bgra_len].as_chunks::<4>();
                                            for chunk in chunks {
                                                rgba.push(chunk[2]); // R
                                                rgba.push(chunk[1]); // G
                                                rgba.push(chunk[0]); // B
                                                rgba.push(255);      // A
                                            }
                                            rgba
                                        } else {
                                            slice.to_vec()
                                        };

                                        let _ = buf.Unlock();

                                        Some(Frame {
                                            data: rgba_data,
                                            width: frame_width,
                                            height: frame_height,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(frame) = frame_opt
                        && let Ok(mut lock) = buffer.lock()
                    {
                        *lock = Some(frame);
                    }

                    let elapsed = start_time.elapsed();
                    if elapsed < frame_duration {
                        thread::sleep(frame_duration - elapsed);
                    }
                }

                Ok(())
            })();

            if capture_result.is_err() {
                running.store(false, Ordering::SeqCst);
            }

            unsafe {
                let _ = MFShutdown();
                CoUninitialize();
            }
        });

        if let Ok(mut lock) = self.thread_handle.lock() {
            *lock = Some(handle);
        }
    }

    fn stop_liveview(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Ok(mut lock) = self.thread_handle.lock()
            && let Some(handle) = lock.take()
        {
            let _ = handle.join();
        }
    }

    fn is_streaming(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
