mod registers;

use registers::*;

pub struct Keypad {
    pub keyinput: KEYINPUT,
    pub keycnt: KEYCNT,
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keyinput: KEYINPUT::empty(),
            keycnt: KEYCNT::empty(),
        }
    }
}
