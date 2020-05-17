mod registers;

use registers::*;
use super::MemoryHandler;
use super::IORegister;

pub struct PPU {
    // Registers
    dispcnt: DISPCNT,
    dispstat: DISPSTAT,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            dispcnt: DISPCNT::new(),
            dispstat: DISPSTAT::new(),
        }
    }
}

impl MemoryHandler for PPU {
    fn read8(&self, addr: u32) -> u8 {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => (self.dispcnt.read() >> 0) as u8,
            0x001 => (self.dispcnt.read() >> 8) as u8,
            0x004 => (self.dispstat.read() >> 0) as u8,
            0x005 => (self.dispstat.read() >> 8) as u8,
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => self.dispcnt.write(0x00FF, (value as u16) << 0),
            0x001 => self.dispcnt.write(0xFF00, (value as u16) << 8),
            0x004 => self.dispstat.write(0x00FF, (value as u16) << 0),
            0x005 => self.dispstat.write(0xFF00, (value as u16) << 8),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
