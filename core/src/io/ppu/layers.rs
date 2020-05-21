use super::{BG01CNT, BG23CNT, Display, OFS, ReferencePointCoord, RotationScalingParameter};
pub trait Layer {
    fn render(&mut self, pixels: &[u16; Display::WIDTH * Display::HEIGHT]);
}

pub struct BG01 {
    pub cnt: BG01CNT,
    pub hofs: OFS,
    pub vofs: OFS,
}

impl BG01 {
    pub fn new() -> BG01 {
        BG01 {
            cnt: BG01CNT::new(),
            hofs: OFS::new(),
            vofs: OFS::new(),
        }
    }
}

impl Layer for BG01 {
    fn render(&mut self, _pixels: &[u16; Display::WIDTH * Display::HEIGHT]) {

    }
}


pub struct BG23 {
    pub cnt: BG23CNT,
    pub hofs: OFS,
    pub vofs: OFS,
    pub pa: RotationScalingParameter,
    pub pb: RotationScalingParameter,
    pub pc: RotationScalingParameter,
    pub pd: RotationScalingParameter,
    pub ref_point_x: ReferencePointCoord,
    pub ref_point_y: ReferencePointCoord,
}

impl BG23 {
    pub fn new() -> BG23 {
        BG23 {
            cnt: BG23CNT::new(),
            hofs: OFS::new(),
            vofs: OFS::new(),
            pa: RotationScalingParameter::new(),
            pb: RotationScalingParameter::new(),
            pc: RotationScalingParameter::new(),
            pd: RotationScalingParameter::new(),
            ref_point_x: ReferencePointCoord::new(),
            ref_point_y: ReferencePointCoord::new(),
        }
    }
}

impl Layer for BG23 {
    fn render(&mut self, _pixels: &[u16; Display::WIDTH * Display::HEIGHT]) {

    }
}
