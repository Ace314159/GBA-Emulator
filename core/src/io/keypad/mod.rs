mod registers;

pub use registers::KEYINPUT;
use registers::*;

pub struct Keypad {
    pub keyinput: KEYINPUT,
    pub keycnt: KEYCNT,
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keyinput: KEYINPUT::all(),
            keycnt: KEYCNT::empty(),
        }
    }

    pub fn press_key(&mut self, key: KEYINPUT) { self.keyinput.remove(key) }
    pub fn release_key(&mut self, key: KEYINPUT) { self.keyinput.insert(key) }

    pub fn interrupt_requested(&self) -> bool {
        if self.keycnt.contains(KEYCNT::IRQ_ENABLE) {
            let irq_keys = self.keycnt - KEYCNT::IRQ_ENABLE - KEYCNT::IRQ_COND_AND;
            if self.keycnt.contains(KEYCNT::IRQ_COND_AND) { irq_keys.bits() & !self.keyinput.bits() == irq_keys.bits() }
            else { irq_keys.bits() & !self.keyinput.bits() != 0 }
        } else { false }
    }
}
