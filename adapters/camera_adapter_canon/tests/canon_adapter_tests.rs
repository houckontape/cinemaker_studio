use camera_domain::{CameraInfo, FrameBuffer};
use camera_ports::{CameraDetector, CameraLiveView, CameraPort};
use camera_adapter_canon::{CanonCameraAdapter, CanonLiveViewAdapter};
use std::sync::{Arc, Mutex};

#[test]
fn test_canon_camera_adapter_traits() {
    let adapter = CanonCameraAdapter::new();
    println!("Call 1 detect: {:?}", CanonCameraAdapter::detect_cameras());
    println!("Call 2 detect: {:?}", CanonCameraAdapter::detect_cameras());
    println!("Call 3 detect: {:?}", CanonCameraAdapter::detect_cameras());

    let info_list: Vec<CameraInfo> = CameraDetector::list_cameras(&adapter);
    println!("Canon cameras detected (CameraDetector): {:?}", info_list);

    let cam_list = CameraPort::list_cameras(&adapter);
    println!("Canon cameras detected (CameraPort): {:?}", cam_list);
}

#[test]
fn test_canon_liveview_adapter_stream() {
    let liveview = CanonLiveViewAdapter::new();
    assert!(!liveview.is_streaming());

    let frame_buffer: FrameBuffer = Arc::new(Mutex::new(None));
    liveview.start_liveview("canon_0", frame_buffer);
    assert!(liveview.is_streaming());

    liveview.stop_liveview();
    assert!(!liveview.is_streaming());
}

#[test]
fn test_canon_property_decoding() {
    use camera_adapter_canon::property_decoder::*;

    assert_eq!(decode_iso(Some(0x00)), "Auto");
    assert_eq!(decode_iso(Some(0x48)), "ISO 100");
    assert_eq!(decode_iso(Some(0x58)), "ISO 400");
    assert_eq!(decode_iso(Some(0x60)), "ISO 800");

    assert_eq!(decode_av(Some(0x20)), "f/2.8");
    assert_eq!(decode_av(Some(0x28)), "f/4.0");
    assert_eq!(decode_av(Some(0x30)), "f/5.6");
    assert_eq!(decode_av(Some(0x38)), "f/8.0");

    assert_eq!(decode_tv(Some(0x68)), "1/60s");
    assert_eq!(decode_tv(Some(0x70)), "1/125s");
    assert_eq!(decode_tv(Some(0x78)), "1/250s");
    assert_eq!(decode_tv(Some(0x04)), "Bulb");

    assert_eq!(decode_exp_comp(Some(0x00)), "0 EV");
    assert_eq!(decode_exp_comp(Some(0x08)), "+1 EV");
    assert_eq!(decode_exp_comp(Some(0xF8)), "-1 EV");

    assert_eq!(decode_white_balance(Some(0)), "Auto");
    assert_eq!(decode_white_balance(Some(1)), "Lumière du jour");
    assert_eq!(decode_white_balance(Some(3)), "Tungstène");

    assert_eq!(decode_battery(Some(100), None), "100%");
    assert_eq!(decode_battery(Some(0xffffffff), None), "Secteur (AC)");
    assert_eq!(decode_battery(None, Some(4)), "Pleine (100%)");

    assert_eq!(decode_ae_mode(Some(0)), "Programme (P)");
    assert_eq!(decode_ae_mode(Some(1)), "Priorité Vitesse (Tv)");
    assert_eq!(decode_ae_mode(Some(2)), "Priorité Ouverture (Av)");
    assert_eq!(decode_ae_mode(Some(3)), "Manuel (M)");
}
