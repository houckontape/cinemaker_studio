#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CameraSettings {
    pub iso: String,
    pub aperture: String,
    pub shutter_speed: String,
    pub exposure_comp: String,
    pub white_balance: String,
    pub battery_level: String,
    pub shooting_mode: String,
}

impl CameraSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.iso.is_empty()
            && self.aperture.is_empty()
            && self.shutter_speed.is_empty()
            && self.exposure_comp.is_empty()
            && self.white_balance.is_empty()
            && self.battery_level.is_empty()
            && self.shooting_mode.is_empty()
    }
}
