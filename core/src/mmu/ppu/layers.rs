use super::{BGCNT, Display};
pub trait Layer {
    fn render(&mut self, pixels: &[u16; Display::WIDTH * Display::HEIGHT]);
}

pub struct BG<T> {
    pub cnt: T,
}

impl<T> BG<T> where T: BGCNT {
    pub fn new() -> BG<T> {
        BG {
            cnt: T::new(),
        }
    }
}

impl<T> Layer for BG<T> {
    fn render(&mut self, _pixels: &[u16; Display::WIDTH * Display::HEIGHT]) {

    }
}
