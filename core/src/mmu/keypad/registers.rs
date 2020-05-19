use crate::mmu::IORegister;

bitflags! {
    pub struct KEYINPUT: u16 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const RIGHT = 1 << 4;
        const LEFT = 1 << 5;
        const UP = 1 << 6;
        const DOWN = 1 << 7;
        const R = 1 << 8;
        const L = 1 << 9;
    }
}

bitflags! {
    pub struct KEYCNT: u16 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const RIGHT = 1 << 4;
        const LEFT = 1 << 5;
        const UP = 1 << 6;
        const DOWN = 1 << 7;
        const R = 1 << 8;
        const L = 1 << 9;
        const IRQ_ENABLE = 1 << 14;
        const IRQ_CONDITION = 1 << 15;
    }
}

impl IORegister for KEYINPUT {
    fn read_low(&self) -> u8 {
        self.bits as u8
    }

    fn read_high(&self) -> u8 {
        (self.bits >> 8) as u8
    }

    fn write_low(&mut self, _value: u8) {}
    fn write_high(&mut self, _value: u8) {}
}

impl IORegister for KEYCNT {
    fn read_low(&self) -> u8 {
        self.bits as u8
    }

    fn read_high(&self) -> u8 {
        (self.bits >> 8) as u8
    }

    fn write_low(&mut self, value: u8) {
        self.bits = self.bits & !0x00FF | (value as u16) & KEYCNT::all().bits;
    }

    fn write_high(&mut self, value: u8) {
        self.bits = self.bits & !0xFF00 | (value as u16) << 8 & KEYCNT::all().bits;
    }
}
