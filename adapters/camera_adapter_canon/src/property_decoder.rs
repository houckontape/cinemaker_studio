use camera_domain::CameraSettings;
use crate::edsdk_sys::*;

/// Lit et décode les réglages de la caméra Canon active.
///
/// # Safety
///
/// `cam_ref` doit être un pointeur `EdsCameraRef` valide et ouvert.
pub unsafe fn read_camera_settings(cam_ref: EdsCameraRef) -> CameraSettings {
    if cam_ref.is_null() {
        return CameraSettings::default();
    }

    let iso_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_ISOSpeed) };
    let av_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_Av) };
    let tv_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_Tv) };
    let exp_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_ExposureCompensation) };
    let wb_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_WhiteBalance) };
    let battery_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_BatteryLevel) };
    let battery_qual = unsafe { get_uint32_prop(cam_ref, kEdsPropID_BatteryQuality) };
    let mut ae_val = unsafe { get_uint32_prop(cam_ref, kEdsPropID_AEMode) };
    if (ae_val.is_none() || ae_val == Some(0))
        && let Some(ae_sel) = unsafe { get_uint32_prop(cam_ref, kEdsPropID_AEModeSelect) }
    {
        ae_val = Some(ae_sel);
    }

    CameraSettings {
        iso: decode_iso(iso_val),
        aperture: decode_av(av_val),
        shutter_speed: decode_tv(tv_val),
        exposure_comp: decode_exp_comp(exp_val),
        white_balance: decode_white_balance(wb_val),
        battery_level: decode_battery(battery_val, battery_qual),
        shooting_mode: decode_ae_mode(ae_val),
    }
}

unsafe fn get_uint32_prop(cam_ref: EdsCameraRef, prop_id: EdsPropertyID) -> Option<u32> {
    let mut data: EdsUInt32 = 0;
    let mut prop_type: EdsUInt32 = 0;
    let mut prop_size: EdsUInt32 = 0;

    let size_err = unsafe {
        EdsGetPropertySize(
            cam_ref,
            prop_id,
            0,
            &mut prop_type as *mut _ as *mut _,
            &mut prop_size as *mut _ as *mut _,
        )
    };

    if size_err == EDS_ERR_OK && prop_size > 0 {
        let err = unsafe {
            EdsGetPropertyData(
                cam_ref,
                prop_id,
                0,
                prop_size.min(std::mem::size_of::<EdsUInt32>() as EdsUInt32),
                &mut data as *mut _ as *mut _,
            )
        };
        if err == EDS_ERR_OK {
            return Some(data);
        }
    }

    // Requête directe de secours
    let err = unsafe {
        EdsGetPropertyData(
            cam_ref,
            prop_id,
            0,
            std::mem::size_of::<EdsUInt32>() as EdsUInt32,
            &mut data as *mut _ as *mut _,
        )
    };
    if err == EDS_ERR_OK {
        Some(data)
    } else {
        None
    }
}

pub fn decode_iso(val: Option<u32>) -> String {
    let Some(v) = val else { return "Auto".to_string() };
    match v {
        0x00 | 0xffffffff => "Auto".to_string(),
        0x28 => "ISO 6".to_string(),
        0x30 => "ISO 12".to_string(),
        0x38 => "ISO 25".to_string(),
        0x40 => "ISO 50".to_string(),
        0x48 => "ISO 100".to_string(),
        0x4b => "ISO 125".to_string(),
        0x4d => "ISO 160".to_string(),
        0x50 => "ISO 200".to_string(),
        0x53 => "ISO 250".to_string(),
        0x55 => "ISO 320".to_string(),
        0x58 => "ISO 400".to_string(),
        0x5b => "ISO 500".to_string(),
        0x5d => "ISO 640".to_string(),
        0x60 => "ISO 800".to_string(),
        0x63 => "ISO 1000".to_string(),
        0x65 => "ISO 1250".to_string(),
        0x68 => "ISO 1600".to_string(),
        0x6b => "ISO 2000".to_string(),
        0x6d => "ISO 2500".to_string(),
        0x70 => "ISO 3200".to_string(),
        0x73 => "ISO 4000".to_string(),
        0x75 => "ISO 5000".to_string(),
        0x78 => "ISO 6400".to_string(),
        0x7b => "ISO 8000".to_string(),
        0x7d => "ISO 10000".to_string(),
        0x80 => "ISO 12800".to_string(),
        0x88 => "ISO 25600".to_string(),
        0x90 => "ISO 51200".to_string(),
        0x98 => "ISO 102400".to_string(),
        dec if (50..=204800).contains(&dec) => format!("ISO {}", dec),
        other => format!("ISO {:#x}", other),
    }
}

pub fn decode_av(val: Option<u32>) -> String {
    let Some(v) = val else { return "f/--".to_string() };
    match v {
        0x00 | 0xffffffff => "f/--".to_string(),
        0x08 => "f/1.0".to_string(),
        0x0B => "f/1.1".to_string(),
        0x0C | 0x0D => "f/1.2".to_string(),
        0x10 => "f/1.4".to_string(),
        0x13 => "f/1.6".to_string(),
        0x14 | 0x15 => "f/1.8".to_string(),
        0x18 => "f/2.0".to_string(),
        0x1B => "f/2.2".to_string(),
        0x1C | 0x1D => "f/2.5".to_string(),
        0x20 => "f/2.8".to_string(),
        0x23 => "f/3.2".to_string(),
        0x24 | 0x25 => "f/3.5".to_string(),
        0x28 => "f/4.0".to_string(),
        0x2B | 0x2C => "f/4.5".to_string(),
        0x2D => "f/5.0".to_string(),
        0x30 => "f/5.6".to_string(),
        0x33 => "f/6.3".to_string(),
        0x34 | 0x35 => "f/7.1".to_string(),
        0x38 => "f/8.0".to_string(),
        0x3B => "f/9.0".to_string(),
        0x3C | 0x3D => "f/10".to_string(),
        0x40 => "f/11".to_string(),
        0x43 | 0x44 => "f/13".to_string(),
        0x45 => "f/14".to_string(),
        0x48 => "f/16".to_string(),
        0x4B => "f/18".to_string(),
        0x4C | 0x4D => "f/20".to_string(),
        0x50 => "f/22".to_string(),
        0x53 => "f/25".to_string(),
        0x54 | 0x55 => "f/29".to_string(),
        0x58 => "f/32".to_string(),
        0x5B | 0x5C => "f/36".to_string(),
        0x5D => "f/40".to_string(),
        0x60 => "f/45".to_string(),
        0x68 => "f/64".to_string(),
        other => format!("f/{:#x}", other),
    }
}

pub fn decode_tv(val: Option<u32>) -> String {
    let Some(v) = val else { return "--s".to_string() };
    match v {
        0x00 | 0xffffffff => "--s".to_string(),
        0x04 | 0x0C => "Bulb".to_string(),
        0x10 => "30\"".to_string(),
        0x13 => "25\"".to_string(),
        0x14 | 0x15 => "20\"".to_string(),
        0x18 => "15\"".to_string(),
        0x1B => "13\"".to_string(),
        0x1C | 0x1D => "10\"".to_string(),
        0x20 => "8\"".to_string(),
        0x23 => "6\"".to_string(),
        0x24 | 0x25 => "5\"".to_string(),
        0x28 => "4\"".to_string(),
        0x2B => "3.2\"".to_string(),
        0x2C | 0x2D => "2.5\"".to_string(),
        0x30 => "2\"".to_string(),
        0x33 => "1.6\"".to_string(),
        0x34 | 0x35 => "1.3\"".to_string(),
        0x38 => "1\"".to_string(),
        0x3B => "0.8\"".to_string(),
        0x3C => "0.7\"".to_string(),
        0x3D => "0.6\"".to_string(),
        0x40 => "0.5\" (1/2)".to_string(),
        0x43 => "0.4\"".to_string(),
        0x44 => "0.3\"".to_string(),
        0x45 | 0x48 => "1/4s".to_string(),
        0x4B => "1/5s".to_string(),
        0x4C | 0x4D => "1/6s".to_string(),
        0x50 => "1/8s".to_string(),
        0x53 | 0x54 => "1/10s".to_string(),
        0x55 => "1/13s".to_string(),
        0x58 => "1/15s".to_string(),
        0x5B | 0x5C => "1/20s".to_string(),
        0x5D => "1/25s".to_string(),
        0x60 => "1/30s".to_string(),
        0x63 => "1/40s".to_string(),
        0x64 => "1/45s".to_string(),
        0x65 => "1/50s".to_string(),
        0x68 => "1/60s".to_string(),
        0x6B => "1/80s".to_string(),
        0x6C => "1/90s".to_string(),
        0x6D => "1/100s".to_string(),
        0x70 => "1/125s".to_string(),
        0x73 => "1/160s".to_string(),
        0x74 => "1/180s".to_string(),
        0x75 => "1/200s".to_string(),
        0x78 => "1/250s".to_string(),
        0x7B => "1/320s".to_string(),
        0x7C => "1/350s".to_string(),
        0x7D => "1/400s".to_string(),
        0x80 => "1/500s".to_string(),
        0x83 => "1/640s".to_string(),
        0x84 => "1/750s".to_string(),
        0x85 => "1/800s".to_string(),
        0x88 => "1/1000s".to_string(),
        0x8B => "1/1250s".to_string(),
        0x8C => "1/1500s".to_string(),
        0x8D => "1/1600s".to_string(),
        0x90 => "1/2000s".to_string(),
        0x93 => "1/2500s".to_string(),
        0x94 => "1/3000s".to_string(),
        0x95 => "1/3200s".to_string(),
        0x98 => "1/4000s".to_string(),
        0x9B => "1/5000s".to_string(),
        0x9C => "1/6000s".to_string(),
        0x9D => "1/6400s".to_string(),
        0xA0 => "1/8000s".to_string(),
        0xA8 => "1/16000s".to_string(),
        other => format!("{:#x}s", other),
    }
}

pub fn decode_exp_comp(val: Option<u32>) -> String {
    let Some(v) = val else { return "0 EV".to_string() };
    match v {
        0x00 => "0 EV".to_string(),
        0x03 => "+1/3 EV".to_string(),
        0x04 => "+1/2 EV".to_string(),
        0x05 => "+2/3 EV".to_string(),
        0x08 => "+1 EV".to_string(),
        0x0B => "+1 1/3 EV".to_string(),
        0x0C => "+1 1/2 EV".to_string(),
        0x0D => "+1 2/3 EV".to_string(),
        0x10 => "+2 EV".to_string(),
        0x13 => "+2 1/3 EV".to_string(),
        0x14 => "+2 1/2 EV".to_string(),
        0x15 => "+2 2/3 EV".to_string(),
        0x18 => "+3 EV".to_string(),
        0x20 => "+4 EV".to_string(),
        0x28 => "+5 EV".to_string(),
        0xFD => "-1/3 EV".to_string(),
        0xFC => "-1/2 EV".to_string(),
        0xFB => "-2/3 EV".to_string(),
        0xF8 => "-1 EV".to_string(),
        0xF5 => "-1 1/3 EV".to_string(),
        0xF4 => "-1 1/2 EV".to_string(),
        0xF3 => "-1 2/3 EV".to_string(),
        0xF0 => "-2 EV".to_string(),
        0xED => "-2 1/3 EV".to_string(),
        0xEC => "-2 1/2 EV".to_string(),
        0xEB => "-2 2/3 EV".to_string(),
        0xE8 => "-3 EV".to_string(),
        0xE0 => "-4 EV".to_string(),
        0xD8 => "-5 EV".to_string(),
        _ => "0 EV".to_string(),
    }
}

pub fn decode_white_balance(val: Option<u32>) -> String {
    let Some(v) = val else { return "Auto".to_string() };
    match v {
        0 => "Auto".to_string(),
        1 => "Lumière du jour".to_string(),
        2 => "Nuageux".to_string(),
        3 => "Tungstène".to_string(),
        4 => "Fluorescent".to_string(),
        5 => "Flash".to_string(),
        6 => "Personnalisé".to_string(),
        8 => "Ombre".to_string(),
        9 => "Température Kelvin".to_string(),
        10 => "PC-1".to_string(),
        11 => "PC-2".to_string(),
        12 => "PC-3".to_string(),
        15 => "Auto (Priorité blanc)".to_string(),
        _ => "Auto".to_string(),
    }
}

pub fn decode_battery(level: Option<u32>, quality: Option<u32>) -> String {
    if let Some(q) = quality
        && q == 0xffffffff
    {
        return "Secteur (AC)".to_string();
    }
    if let Some(v) = level {
        if v == 0xffffffff {
            return "Secteur (AC)".to_string();
        }
        if v > 4 && v <= 100 {
            return format!("{}%", v);
        }
        match v {
            4 => return "Pleine (100%)".to_string(),
            3 => return "Bonne (75%)".to_string(),
            2 => return "Moyenne (50%)".to_string(),
            1 => return "Faible (25%)".to_string(),
            _ => {}
        }
    }
    if let Some(q) = quality {
        match q {
            4..=6 => return "Pleine (100%)".to_string(),
            3 => return "Bonne (75%)".to_string(),
            2 => return "Moyenne (50%)".to_string(),
            1 => return "Faible (25%)".to_string(),
            pct if (5..=100).contains(&pct) => return format!("{}%", pct),
            _ => {}
        }
    }
    "100%".to_string()
}

pub fn decode_ae_mode(val: Option<u32>) -> String {
    let Some(v) = val else { return "Manuel (M)".to_string() };
    match v {
        0 => "Programme (P)".to_string(),
        1 => "Priorité Vitesse (Tv)".to_string(),
        2 => "Priorité Ouverture (Av)".to_string(),
        3 => "Manuel (M)".to_string(),
        4 => "Pose B (Bulb)".to_string(),
        5 => "A-DEP".to_string(),
        6 => "DEP".to_string(),
        7 => "Personnalisé 1 (C1)".to_string(),
        9 => "Scène Intelligente Auto".to_string(),
        10 => "Créatif Auto".to_string(),
        11 => "Personnalisé 2 (C2)".to_string(),
        12 => "Personnalisé 3 (C3)".to_string(),
        13 => "Portrait".to_string(),
        14 => "Paysage".to_string(),
        15 => "Macro".to_string(),
        16 => "Sport".to_string(),
        17 => "Scène Nuit".to_string(),
        20 => "Flash Off".to_string(),
        22 => "Vidéo Standard".to_string(),
        23 => "Vidéo Manuel".to_string(),
        _ => "Automatique".to_string(),
    }
}
