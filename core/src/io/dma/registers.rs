use super::super::{Scheduler, IORegister};

pub struct Address {
    pub addr: u32,
    byte3_mask: u32,
}

impl Address {
    pub fn new(any_memory: bool) -> Address {
        Address {
            addr: 0,
            byte3_mask: if any_memory { 0x0FF0_0000 } else { 0x07F0_0000 },
        }
    }
}

impl IORegister for Address {
    fn read(&self, _byte: u8) -> u8 { 0 }

    fn write(&mut self, __scheduler: &mut Scheduler, byte: u8, value: u8) {
        let mask = 0xFF << (8 * byte);
        match byte {
            0 ..= 2 => self.addr = self.addr & !mask | (value as u32) << (8 * byte) & mask,
            3 => self.addr = self.addr & !mask | (value as u32) << (8 * byte) & self.byte3_mask,
            _ => unreachable!(),
        }
    }
}

pub struct WordCount {
    pub count: u16,
    max: u16,
}

impl WordCount {
    pub fn new(is_16bit: bool) -> WordCount {
        WordCount {
            count: 0,
            max: if is_16bit { 0xFFFF } else { 0x3FFF },
        }
    }

    pub fn get_max(&self) -> u32 { self.max as u32 }
}

impl IORegister for WordCount {
    fn read(&self, _byte: u8) -> u8 { 0 }

    fn write(&mut self, _scheduler: &mut Scheduler, byte: u8, value: u8) {
        match byte {
            0 => self.count = self.count & !0x00FF | value as u16,
            1 => self.count = self.count & !0xFF00 | (value as u16) << 8 & self.max,
            _ => unreachable!(),
        }
    }
}

pub struct DMACNT {
    pub dest_addr_ctrl: u8,
    pub src_addr_ctrl: u8,
    pub repeat: bool,
    pub transfer_32: bool,
    pub game_pak_drq: bool,
    pub start_timing: u8,
    pub irq: bool,
    pub enable: bool,

    is_dma3: bool,
}

impl DMACNT {
    pub fn new(is_dma3: bool) -> DMACNT {
        DMACNT {
            dest_addr_ctrl: 0,
            src_addr_ctrl: 0,
            repeat: false,
            transfer_32: false,
            game_pak_drq: false,
            start_timing: 0,
            irq: false,
            enable: false,
        
            is_dma3,
        }
    }
}

impl IORegister for DMACNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.src_addr_ctrl & 0x1) << 7 | self.dest_addr_ctrl << 5,
            1 => (self.enable as u8) << 7 | (self.irq as u8) << 6 | self.start_timing << 4 | (self.game_pak_drq as u8) |
                (self.transfer_32 as u8) << 2 | (self.repeat as u8) << 1 | self.src_addr_ctrl >> 1,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, _scheduler: &mut Scheduler, byte: u8, value: u8) {
        match byte {
            0 => {
                self.src_addr_ctrl = self.src_addr_ctrl & !0x1 | value >> 7 & 0x1;
                self.dest_addr_ctrl = value >> 5 & 0x3;
            },
            1 => {
                self.enable = value >> 7 & 0x1 != 0;
                self.irq = value >> 6 & 0x1 != 0;
                self.start_timing = value >> 4 & 0x3;
                if self.is_dma3 { self.game_pak_drq = value >> 3 & 0x1 != 0 }
                self.transfer_32 = value >> 2 & 0x1 != 0;
                self.repeat = value >> 1 & 0x1 != 0;
                self.src_addr_ctrl = self.src_addr_ctrl & !0x2 | value << 1 & 0x2;
            },
            _ => unreachable!(),
        }
    }
}
