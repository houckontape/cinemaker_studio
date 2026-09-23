use camera_domain::{Camera, CameraKind};
use camera_ports::CameraPort;
use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;

#[derive(Clone, Default)]
pub struct CameraAdapter;

impl CameraAdapter {
    pub fn new() -> Self {
        unsafe {
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        Self
    }
}

impl CameraPort for CameraAdapter {
    fn list_cameras(&self) -> Vec<Camera> {
        unsafe {
            let mut attributes: Option<IMFAttributes> = None;
            if MFCreateAttributes(&mut attributes, 1).is_err() {
                return Vec::new();
            }
            let Some(attributes) = attributes else {
                return Vec::new();
            };

            if attributes
                .SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .is_err()
            {
                return Vec::new();
            }

            let mut devices: *mut Option<IMFActivate> = std::ptr::null_mut();
            let mut count = 0;

            if MFEnumDeviceSources(&attributes, &mut devices, &mut count).is_err() {
                return Vec::new();
            }

            if devices.is_null() || count == 0 {
                return Vec::new();
            }

            let slice = std::slice::from_raw_parts(devices, count as usize);
            let mut cameras = Vec::new();

            for (i, dev_opt) in slice.iter().enumerate() {
                if let Some(dev) = dev_opt {
                    let mut name_ptr = PWSTR::null();
                    let mut len = 0;
                    let name = if dev
                        .GetAllocatedString(&MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &mut name_ptr, &mut len)
                        .is_ok()
                    {
                        let s = String::from_utf16_lossy(std::slice::from_raw_parts(name_ptr.0, len as usize));
                        CoTaskMemFree(Some(name_ptr.0 as _));
                        s
                    } else {
                        format!("Camera {}", i)
                    };

                    let mut sym_ptr = PWSTR::null();
                    let mut sym_len = 0;
                    let sym_link = if dev
                        .GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                            &mut sym_ptr,
                            &mut sym_len,
                        )
                        .is_ok()
                    {
                        let s = String::from_utf16_lossy(std::slice::from_raw_parts(sym_ptr.0, sym_len as usize));
                        CoTaskMemFree(Some(sym_ptr.0 as _));
                        s
                    } else {
                        String::new()
                    };

                    let name_lower = name.to_lowercase();
                    let sym_lower = sym_link.to_lowercase();

                    let kind = if name_lower.contains("internal")
                        || name_lower.contains("integrated")
                        || name_lower.contains("built-in")
                        || name_lower.contains("builtin")
                        || name_lower.contains("facetime")
                        || name_lower.contains("webcam")
                        || name_lower.contains("easycamera")
                        || sym_lower.contains("internal")
                        || sym_lower.contains("integrated")
                        || sym_lower.contains("acpi")
                    {
                        CameraKind::Internal
                    } else {
                        CameraKind::Usb
                    };

                    cameras.push(Camera::new(i.to_string(), name, kind));
                }
            }

            CoTaskMemFree(Some(devices as _));

            cameras
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_hardware_cameras() {
        let adapter = CameraAdapter::new();
        let cameras = adapter.list_cameras();
        println!("Detected cameras: {:#?}", cameras);
    }
}
