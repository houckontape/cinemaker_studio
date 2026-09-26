use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use log::{error, info, warn};

use camera_domain::{CameraSettings, Frame, FrameBuffer};
use camera_ports::{CameraLiveView, CameraStream};

use crate::canon_adapter::{CanonSdk, LIVEVIEW_ACTIVE};
use crate::edsdk_sys::*;
use crate::property_decoder::read_camera_settings;

pub struct CanonLiveViewAdapter {
    is_running: Arc<AtomicBool>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    latest_frame: Arc<Mutex<Option<Frame>>>,
    settings: Arc<Mutex<CameraSettings>>,
}

impl Default for CanonLiveViewAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonLiveViewAdapter {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            thread_handle: Mutex::new(None),
            latest_frame: Arc::new(Mutex::new(None)),
            settings: Arc::new(Mutex::new(CameraSettings::default())),
        }
    }

    pub fn get_settings(&self) -> CameraSettings {
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn parse_camera_index(camera_id: &str) -> usize {
        if let Some(stripped) = camera_id.strip_prefix("canon_") {
            stripped.parse::<usize>().unwrap_or(0)
        } else if let Some(stripped) = camera_id.strip_prefix("canon:") {
            stripped.parse::<usize>().unwrap_or(0)
        } else {
            camera_id.parse::<usize>().unwrap_or(0)
        }
    }

    /// Boucle principale de capture du flux LiveView (exécutée dans un thread dédié).
    ///
    /// Cette méthode réalise :
    /// 1. L'initialisation du runtime COM Windows (STA requis par l'EDSDK Canon).
    /// 2. L'accès à la caméra sélectionnée via `EdsGetChildAtIndex`.
    /// 3. L'ouverture de la session avec gestion de retries en cas de bus USB occupé.
    /// 4. L'initialisation du mode EVF (`kEdsPropID_Evf_Mode`) et l'activation de la sortie PC (`kEdsPropID_Evf_OutputDevice`).
    /// 5. La boucle de capture continue : création du flux mémoire, téléchargement de l'image EVF (`EdsDownloadEvfImage`),
    ///    décompression JPEG en mémoire vers RGBA et transmission au buffer partagé de l'application UI.
    /// 6. Le rafraîchissement périodique des paramètres matériels (ISO, vitesse, ouverture, batterie).
    /// 7. La fermeture propre de la session et la désactivation de la sortie LiveView lors de l'arrêt.
    fn run_liveview_loop(
        camera_index: usize,
        is_running: Arc<AtomicBool>,
        frame_buffer: Option<FrameBuffer>,
        latest_frame: Arc<Mutex<Option<Frame>>>,
        settings_store: Arc<Mutex<CameraSettings>>,
    ) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        if let Err(e) = CanonSdk::init() {
            error!("Erreur CanonSdk::init dans le thread liveview: {}", e);
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        let mut camera_list: EdsCameraListRef = std::ptr::null_mut();
        let err = unsafe { EdsGetCameraList(&mut camera_list) };
        if err != EDS_ERR_OK || camera_list.is_null() {
            error!("Erreur EdsGetCameraList: {:#x}", err);
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        let mut cam_ref: EdsCameraRef = std::ptr::null_mut();
        let err = unsafe { EdsGetChildAtIndex(camera_list, camera_index as EdsInt32, &mut cam_ref) };
        if err != EDS_ERR_OK || cam_ref.is_null() {
            error!("Impossible d'obtenir la caméra Canon à l'index {}: {:#x}", camera_index, err);
            unsafe { EdsRelease(camera_list) };
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        // Ouverture de la session avec boucle de retry en cas de bus occupé
        let mut open_err = unsafe { EdsOpenSession(cam_ref) };
        let mut retries = 0;
        while (open_err == EDS_ERR_DEVICE_BUSY || open_err == EDS_ERR_COMM_PORT_IS_IN_USE) && retries < 15 {
            thread::sleep(Duration::from_millis(50));
            unsafe {
                let _ = EdsGetEvent();
                open_err = EdsOpenSession(cam_ref);
            }
            retries += 1;
        }

        if open_err != EDS_ERR_OK && open_err != EDS_ERR_SESSION_ALREADY_OPEN {
            error!("Erreur EdsOpenSession: {:#x}", open_err);
            unsafe {
                EdsRelease(cam_ref);
                EdsRelease(camera_list);
            }
            is_running.store(false, Ordering::SeqCst);
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Com::CoUninitialize;
                CoUninitialize();
            }
            return;
        }

        // Pompage initial des événements pour recevoir la configuration complète de l'appareil
        for _ in 0..8 {
            unsafe {
                let _ = EdsGetEvent();
            }
            thread::sleep(Duration::from_millis(25));
        }

        // Lecture des réglages initiaux
        let initial_settings = unsafe { read_camera_settings(cam_ref) };
        if let Ok(mut lock) = settings_store.lock() {
            *lock = initial_settings;
        }

        // Configuration du mode LiveView (Evf_Mode = 1) : obligatoire sur de nombreux boîtiers Canon EOS
        // pour autoriser le basculement du capteur et l'activation du miroir en mode vidéo/LiveView
        let evf_mode: EdsUInt32 = 1;
        let set_mode_err = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_Mode,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &evf_mode as *const _ as *const _,
            )
        };
        if set_mode_err != EDS_ERR_OK {
            warn!("Avertissement EdsSetPropertyData(kEdsPropID_Evf_Mode): {:#x}", set_mode_err);
        }

        // Configuration du zoom EVF en mode Fit (1x : image complète non recadrée)
        let evf_zoom: EdsUInt32 = kEdsEvfZoom_Fit;
        let _ = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_Zoom,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &evf_zoom as *const _ as *const _,
            )
        };

        // Activation de la sortie LiveView vers le PC (kEdsEvfOutputDevice_PC)
        // Note : sur les appareils récents, la combinaison (TFT | PC) ou PC seul bascule le flux EVF.
        let mut retry_count = 0;
        loop {
            let mut device: EdsUInt32 = 0;
            let _ = unsafe {
                EdsGetPropertyData(
                    cam_ref,
                    kEdsPropID_Evf_OutputDevice,
                    0,
                    std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                    &mut device as *mut _ as *mut _,
                )
            };

            device |= kEdsEvfOutputDevice_PC;
            let set_dev_err = unsafe {
                EdsSetPropertyData(
                    cam_ref,
                    kEdsPropID_Evf_OutputDevice,
                    0,
                    std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                    &device as *const _ as *const _,
                )
            };

            if set_dev_err == EDS_ERR_OK || retry_count >= 10 {
                if set_dev_err != EDS_ERR_OK {
                    let pc_only: EdsUInt32 = kEdsEvfOutputDevice_PC;
                    let _ = unsafe {
                        EdsSetPropertyData(
                            cam_ref,
                            kEdsPropID_Evf_OutputDevice,
                            0,
                            std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                            &pc_only as *const _ as *const _,
                        )
                    };
                }
                break;
            }

            unsafe {
                let _ = EdsGetEvent();
            }
            thread::sleep(Duration::from_millis(50));
            retry_count += 1;
        }

        // Attente et pompage pour la stabilisation du miroir et du capteur en mode LiveView
        for _ in 0..5 {
            unsafe {
                let _ = EdsGetEvent();
            }
            thread::sleep(Duration::from_millis(30));
        }

        info!("Canon LiveView démarré avec succès pour l'index {}", camera_index);
        LIVEVIEW_ACTIVE.store(true, Ordering::SeqCst);

        // Maintien de l'appareil éveillé (évite le mode veille ou coupure d'inactivité)
        unsafe {
            let _ = EdsSendCommand(cam_ref, kEdsCameraCommand_ExtendShutDownTimer, 0);
        }

        let mut frame_count: u64 = 0;

        while is_running.load(Ordering::SeqCst) {
            // Traitement des messages et événements EDSDK requis pour le maintien du flux USB
            unsafe {
                let _ = EdsGetEvent();
            }

            // Réinitialisation périodique du timer de veille (toutes les ~60 trames / ~1-2s)
            if frame_count.is_multiple_of(60) {
                unsafe {
                    let _ = EdsSendCommand(cam_ref, kEdsCameraCommand_ExtendShutDownTimer, 0);
                }
            }

            // Étape 1 : Création d'un flux mémoire temporaire pour recevoir la trame compressée JPEG
            let mut stream_ref: EdsStreamRef = std::ptr::null_mut();
            let err = unsafe { EdsCreateMemoryStream(0, &mut stream_ref) };
            if err == EDS_ERR_OK && !stream_ref.is_null() {
                // Étape 2 : Création de la référence d'image EVF associée au flux
                let mut evf_image_ref: EdsEvfImageRef = std::ptr::null_mut();
                let err = unsafe { EdsCreateEvfImageRef(stream_ref, &mut evf_image_ref) };
                if err == EDS_ERR_OK && !evf_image_ref.is_null() {
                    // Étape 3 : Téléchargement de l'image LiveView actuelle depuis la caméra Canon
                    let dl_err = unsafe { EdsDownloadEvfImage(cam_ref, evf_image_ref) };
                    if dl_err == EDS_ERR_OK {
                        let mut ptr: *mut std::os::raw::c_void = std::ptr::null_mut();
                        let mut len: EdsUInt64 = 0;

                        let ptr_err = unsafe { EdsGetPointer(stream_ref, &mut ptr) };
                        let len_err = unsafe { EdsGetLength(stream_ref, &mut len) };

                        // Étape 4 : Extraction du buffer JPEG brut et conversion en image RGBA
                        if ptr_err == EDS_ERR_OK && len_err == EDS_ERR_OK && !ptr.is_null() && len > 0 {
                            let jpeg_data = unsafe {
                                std::slice::from_raw_parts(ptr as *const u8, len as usize)
                            };

                            if let Ok(img) = image::load_from_memory(jpeg_data) {
                                let rgba = img.to_rgba8();
                                let width = rgba.width();
                                let height = rgba.height();
                                let raw = rgba.into_raw();

                                // Étape 5 : Publication de la trame décodée dans le FrameBuffer de l'UI
                                if let Some(ref fb) = frame_buffer
                                    && let Ok(mut lock) = fb.lock()
                                {
                                    *lock = Some(Frame {
                                        data: raw.clone(),
                                        width,
                                        height,
                                    });
                                }

                                if let Ok(mut lock) = latest_frame.lock() {
                                    *lock = Some(Frame {
                                        data: raw,
                                        width,
                                        height,
                                    });
                                }

                                frame_count += 1;
                                // Étape 6 : Rafraîchissement périodique des réglages (toutes les ~30 images / ~1s)
                                if frame_count.is_multiple_of(30) {
                                    let current_settings = unsafe { read_camera_settings(cam_ref) };
                                    if let Ok(mut lock) = settings_store.lock() {
                                        *lock = current_settings;
                                    }
                                }
                            }
                        }
                    } else if dl_err == EDS_ERR_OBJECT_NOTREADY || dl_err == EDS_ERR_DEVICE_BUSY {
                        // Capteur ou mémoire temporairement non prêts, pause courte avant nouvelle tentative
                        thread::sleep(Duration::from_millis(20));
                    } else {
                        warn!("EdsDownloadEvfImage code: {:#x}", dl_err);
                        thread::sleep(Duration::from_millis(30));
                    }

                    // Libération de la structure d'image EVF
                    unsafe { EdsRelease(evf_image_ref) };
                }

                // Libération du flux mémoire
                unsafe { EdsRelease(stream_ref) };
            }

            // Fréquence cible d'environ ~60 FPS maximum pour le LiveView
            thread::sleep(Duration::from_millis(16));
        }

        // Étape 7 : Désactivation de la sortie PC à la fin du flux
        let mut device: EdsUInt32 = 0;
        let _ = unsafe {
            EdsGetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &mut device as *mut _ as *mut _,
            )
        };
        device &= !kEdsEvfOutputDevice_PC;
        let _ = unsafe {
            EdsSetPropertyData(
                cam_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                std::mem::size_of::<EdsUInt32>() as EdsUInt32,
                &device as *const _ as *const _,
            )
        };

        // Étape 8 : Fermeture de session et libération de tous les handles SDK
        unsafe {
            EdsCloseSession(cam_ref);
            EdsRelease(cam_ref);
            EdsRelease(camera_list);
        }

        LIVEVIEW_ACTIVE.store(false, Ordering::SeqCst);

        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Com::CoUninitialize;
            CoUninitialize();
        }

        info!("Canon LiveView arrêté.");
    }
}

impl CameraLiveView for CanonLiveViewAdapter {
    fn start_liveview(&self, camera_id: &str, frame_buffer: FrameBuffer) {
        self.stop_liveview();

        let camera_index = Self::parse_camera_index(camera_id);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let latest_frame = Arc::clone(&self.latest_frame);
        let settings_store = Arc::clone(&self.settings);

        let handle = thread::spawn(move || {
            Self::run_liveview_loop(
                camera_index,
                is_running,
                Some(frame_buffer),
                latest_frame,
                settings_store,
            );
        });

        if let Ok(mut lock) = self.thread_handle.lock() {
            *lock = Some(handle);
        }
    }

    fn stop_liveview(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            let handle = if let Ok(mut lock) = self.thread_handle.lock() {
                lock.take()
            } else {
                None
            };
            if let Some(handle) = handle {
                let _ = handle.join();
            }
        }
    }

    fn is_streaming(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn get_current_settings(&self) -> Option<CameraSettings> {
        let s = self.get_settings();
        if !s.is_empty() {
            Some(s)
        } else {
            None
        }
    }
}

impl CameraStream for CanonLiveViewAdapter {
    fn start_stream(&self, camera_id: &str) {
        if self.is_streaming() {
            self.stop_stream();
        }

        let camera_index = Self::parse_camera_index(camera_id);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);
        let latest_frame = Arc::clone(&self.latest_frame);
        let settings_store = Arc::clone(&self.settings);

        let handle = thread::spawn(move || {
            Self::run_liveview_loop(
                camera_index,
                is_running,
                None,
                latest_frame,
                settings_store,
            );
        });

        if let Ok(mut lock) = self.thread_handle.lock() {
            *lock = Some(handle);
        }
    }

    fn get_frame(&self) -> Option<Frame> {
        if let Ok(guard) = self.latest_frame.lock() {
            guard.clone()
        } else {
            None
        }
    }

    fn stop_stream(&self) {
        self.stop_liveview();
    }
}

pub type LiveViewStream = CanonLiveViewAdapter;
