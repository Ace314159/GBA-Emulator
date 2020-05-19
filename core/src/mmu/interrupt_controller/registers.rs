use crate::mmu::IORegister;

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
    fn read_low(&self) -> u8 {
        self.bits as u8
    }

    fn read_high(&self) -> u8 {
        (self.bits >> 8) as u8
    }

    fn write_low(&mut self, value: u8) {
        self.bits = self.bits & !0x00FF | (value as u16) & InterruptEnable::all().bits;
    }

    fn write_high(&mut self, value: u8) {
        self.bits = self.bits & !0xFF0 | (value as u16) << 8 & InterruptEnable::all().bits;
    }
}

impl IORegister for InterruptMasterEnable {
    fn read_low(&self) -> u8 {
        self.bits as u8
    }

    fn read_high(&self) -> u8 {
        (self.bits >> 8) as u8
    }

    fn write_low(&mut self, value: u8) {
        self.bits = self.bits & !0x00FF | (value as u16) & InterruptMasterEnable::all().bits;
    }

    fn write_high(&mut self, value: u8) {
        self.bits = self.bits & !0xFF0 | (value as u16) << 8 & InterruptMasterEnable::all().bits;
    }
}

impl IORegister for InterruptRequest {
    fn read_low(&self) -> u8 {
        self.bits as u8
    }

    fn read_high(&self) -> u8 {
        (self.bits >> 8) as u8
    }

    fn write_low(&mut self, value: u8) {
        self.bits = self.bits & !0x00FF | (value as u16) & InterruptRequest::all().bits;
    }

    fn write_high(&mut self, value: u8) {
        self.bits = self.bits & !0xFF0 | (value as u16) << 8 & InterruptRequest::all().bits;
    }
}
