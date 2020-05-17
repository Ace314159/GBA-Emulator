mod registers;

use registers::*;
use super::MemoryHandler;
use super::IORegister;

pub struct PPU {
    // Registers
    dispcnt: DISPCNT,
    green_swap: bool,
    dispstat: DISPSTAT,
    vcount: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            dispcnt: DISPCNT::new(),
            green_swap: false,
            dispstat: DISPSTAT::new(),
            vcount: 0, 
        }
    }
}

impl MemoryHandler for PPU {
    fn read8(&self, addr: u32) -> u8 {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => (self.dispcnt.read() >> 0) as u8,
            0x001 => (self.dispcnt.read() >> 8) as u8,
            0x002 => self.green_swap as u8,
            0x003 => 0, // Unused area of Green Swap
            0x004 => (self.dispstat.read() >> 0) as u8,
            0x005 => (self.dispstat.read() >> 8) as u8,
            0x006 => self.vcount as u8,
            0x007 => 0, // Unused area of VCOUNT
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => self.dispcnt.write(0x00FF, (value as u16) << 0),
            0x001 => self.dispcnt.write(0xFF00, (value as u16) << 8),
            0x002 => self.green_swap = value & 0x1 != 0,
            0x003 => {},
            0x004 => self.dispstat.write(0x00FF, (value as u16) << 0),
            0x005 => self.dispstat.write(0xFF00, (value as u16) << 8),
            0x006 => {},
            0x007 => {},
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
