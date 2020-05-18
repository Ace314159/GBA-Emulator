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
    fn read(&self) -> u16 {
        self.bits
    }

    fn write(&mut self, _mask: u16, _value: u16) {}
}

impl IORegister for KEYCNT {
    fn read(&self) -> u16 {
        self.bits
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask;
        self.bits = value & KEYCNT::all().bits();
    }
}
