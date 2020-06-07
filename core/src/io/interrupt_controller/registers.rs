use crate::io::IORegister;

bitflags! {
    pub struct InterruptEnable: u16 {
        const VBLANK = 1 << 0;
        const HBLANK = 1 << 1;
        const VCOUNTER_MATCH = 1 << 2;
        const TIMER0_OVERFLOW = 1 << 3;
        const TIMER1_OVERFLOW = 1 << 4;
        const TIMER2_OVERFLOW = 1 << 5;
        const TIMER3_OVERFLOW = 1 << 6;
        const SERIAL = 1 << 7;
        const DMA0 = 1 << 8;
        const DMA1 = 1 << 9;
        const DMA2 = 1 << 10;
        const DMA3 = 1 << 11;
        const KEYPAD = 1 << 12;
        const GAME_PAK = 1 << 13;
    }
}

bitflags! {
    pub struct InterruptMasterEnable: u16 {
        const ENABLE = 1 << 0;
    }
}

bitflags! {
    pub struct InterruptRequest: u16 {
        const VBLANK = 1 << 0;
        const HBLANK = 1 << 1;
        const VCOUNTER_MATCH = 1 << 2;
        const TIMER0_OVERFLOW = 1 << 3;
        const TIMER1_OVERFLOW = 1 << 4;
        const TIMER2_OVERFLOW = 1 << 5;
        const TIMER3_OVERFLOW = 1 << 6;
        const SERIAL = 1 << 7;
        const DMA0 = 1 << 8;
        const DMA1 = 1 << 9;
        const DMA2 = 1 << 10;
        const DMA3 = 1 << 11;
        const KEYPAD = 1 << 12;
        const GAME_PAK = 1 << 13;
    }
}

impl IORegister for InterruptEnable {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bits as u8,
            1 => (self.bits >> 8) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.bits = self.bits & !0x00FF | (value as u16) & InterruptEnable::all().bits,
            1 => self.bits = self.bits & !0xFF00 | (value as u16) << 8 & InterruptEnable::all().bits,
            _ => unreachable!(),
        }
    }
}

impl IORegister for InterruptMasterEnable {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bits as u8,
            1 => (self.bits >> 8) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.bits = self.bits & !0x00FF | (value as u16) & InterruptMasterEnable::all().bits,
            1 => self.bits = self.bits & !0xFF00 | (value as u16) << 8 & InterruptMasterEnable::all().bits,
            _ => unreachable!(),
        }
    }
}

impl IORegister for InterruptRequest {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bits as u8,
            1 => (self.bits >> 8) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.bits = self.bits & !((value as u16) << 0),
            1 => self.bits = self.bits & !((value as u16) << 8),
            _ => unreachable!(),
        }
    }
}
