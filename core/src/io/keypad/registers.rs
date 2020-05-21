use crate::io::IORegister;

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
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bits as u8,
            1 => (self.bits >> 8) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, _byte: u8, _value: u8) {}
}

impl IORegister for KEYCNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bits as u8,
            1 => (self.bits >> 8) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte:u8, value: u8) {
        match byte {
            0 => self.bits = self.bits & !0x00FF | (value as u16) & KEYCNT::all().bits,
            1 => self.bits = self.bits & !0xFF00 | (value as u16) << 8 & KEYCNT::all().bits,
            _ => panic!("Invalid Byte!"),
        }
    }
}
