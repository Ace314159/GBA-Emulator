use super::{BGCNT, Display, OFS};
pub trait Layer {
    fn render(&mut self, pixels: &[u16; Display::WIDTH * Display::HEIGHT]);
}

pub struct BG<T> {
    pub cnt: T,
    pub hofs: OFS,
    pub vofs: OFS,
}

impl<T> BG<T> where T: BGCNT {
    pub fn new() -> BG<T> {
        BG {
            cnt: T::new(),
            hofs: OFS::new(),
            vofs: OFS::new(),
        }
    }
}

impl<T> Layer for BG<T> {
    fn render(&mut self, _pixels: &[u16; Display::WIDTH * Display::HEIGHT]) {

    }
}
