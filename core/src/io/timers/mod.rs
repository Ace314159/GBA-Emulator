mod registers;

use super::MemoryHandler;
use super::IORegister;

use registers::*;

pub struct Timers {
    timers: [Timer; 4],
}

impl MemoryHandler for Timers {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x4000100 => (self.timers[0].reload >> 0) as u8,
            0x4000101 => (self.timers[0].reload >> 8) as u8,
            0x4000102 => self.timers[0].cnt.read(0),
            0x4000103 => self.timers[0].cnt.read(1),
            0x4000104 => (self.timers[1].reload >> 0) as u8,
            0x4000105 => (self.timers[1].reload >> 8) as u8,
            0x4000106 => self.timers[1].cnt.read(0),
            0x4000107 => self.timers[1].cnt.read(1),
            0x4000108 => (self.timers[2].reload >> 0) as u8,
            0x4000109 => (self.timers[2].reload >> 8) as u8,
            0x400010A => self.timers[2].cnt.read(0),
            0x400010B => self.timers[2].cnt.read(1),
            0x400010C => (self.timers[3].reload >> 0) as u8,
            0x400010D => (self.timers[3].reload >> 8) as u8,
            0x400010E => self.timers[3].cnt.read(0),
            0x400010F => self.timers[3].cnt.read(1),
            _ => unreachable!(),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x4000100 => self.timers[0].reload = self.timers[0].reload & !0x00FF | (value << 0) as u16,
            0x4000101 => self.timers[0].reload = self.timers[0].reload & !0xFF00 | (value << 8) as u16,
            0x4000102 => self.timers[0].cnt.write(0, value),
            0x4000103 => self.timers[0].cnt.write(1, value),
            0x4000104 => self.timers[1].reload = self.timers[1].reload & !0x00FF | (value << 0) as u16,
            0x4000105 => self.timers[1].reload = self.timers[1].reload & !0xFF00 | (value << 8) as u16,
            0x4000106 => self.timers[1].cnt.write(0, value),
            0x4000107 => self.timers[1].cnt.write(1, value),
            0x4000108 => self.timers[2].reload = self.timers[2].reload & !0x00FF | (value << 0) as u16,
            0x4000109 => self.timers[2].reload = self.timers[2].reload & !0xFF00 | (value << 8) as u16,
            0x400010A => self.timers[2].cnt.write(0, value),
            0x400010B => self.timers[2].cnt.write(1, value),
            0x400010C => self.timers[3].reload = self.timers[3].reload & !0x00FF | (value << 0) as u16,
            0x400010D => self.timers[3].reload = self.timers[3].reload & !0xFF00 | (value << 8) as u16,
            0x400010E => self.timers[3].cnt.write(0, value),
            0x400010F => self.timers[3].cnt.write(1, value),
            _ => unreachable!(),
        }
    } 
}

impl Timers {
    pub fn new() -> Timers {
        Timers {
            timers: [Timer::new(); 4],
        }
    }
}

pub struct Timer {
    pub reload: u16,
    pub cnt: TMCNT,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            reload: 0,
            cnt: TMCNT::new(),
        }
    }
}
