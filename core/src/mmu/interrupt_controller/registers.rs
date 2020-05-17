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
    fn read(&self) -> u16 {
        self.bits
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask & InterruptEnable::all().bits();
        self.bits = self.bits & !mask | value;
    }
}

impl IORegister for InterruptMasterEnable {
    fn read(&self) -> u16 {
        self.bits
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask & InterruptMasterEnable::all().bits();
        self.bits = self.bits & !mask | value;
    }
}

impl IORegister for InterruptRequest {
    fn read(&self) -> u16 {
        self.bits
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask & InterruptRequest::all().bits();
        self.bits = self.bits & !value; // Acknowledge interrupt by clearing bits
    }
}
