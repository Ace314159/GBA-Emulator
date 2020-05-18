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

    // Palettes
    bg_colors: [u16; 0x100],
    obj_colors: [u16; 0x100],
    // VRAM
    vram: [u8; 0x18000],

    // Important Rendering Variables
    dot: u16,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            // Registers
            dispcnt: DISPCNT::new(),
            green_swap: false,
            dispstat: DISPSTAT::new(),
            vcount: 0, 

            // Palettes
            bg_colors: [0; 0x100],
            obj_colors: [0; 0x100],
            // VRAM
            vram: [0; 0x18000],

            // Important Rendering Variables
            dot: 0,
        }
    }

    pub fn read_palette_ram(&self, addr: u32) -> u8 {
        let addr = (addr & 0x3FF) as usize;
        let colors = if addr < 0x200 { &self.bg_colors } else { &self.obj_colors };
        let index = (addr & 0xFF) / 2;
        if addr % 2 == 0 {
            (colors[index] >> 0) as u8
        } else {
            (colors[index] >> 8) as u8
        }
    }

    pub fn write_palette_ram(&mut self, addr: u32, value: u8) {
        let addr = (addr & 0x3FF) as usize;
        let colors = if addr < 0x200 { &mut self.bg_colors } else { &mut self.obj_colors };
        let index = (addr & 0xFF) / 2;
        if addr % 2 == 0 {
            colors[index] = colors[index] & !0x00FF | (value as u16) << 0;
        } else {
            colors[index] = colors[index] & !0xFF00 | (value as u16) << 8;
        }
    }

    pub fn read_vram(&self, addr: u32) -> u8 {
        self.vram[(addr - 0x06000000) as usize]
    }

    pub fn write_vram(&mut self, addr: u32, value: u8) {
        self.vram[(addr - 0x06000000) as usize] = value;
    }

    pub fn emulate_dot(&mut self) {
        if self.dot < 240 { // Visible
            self.dispstat.flags.remove(DISPSTATFlags::HBLANK);
        } else { // HBlank
            self.dispstat.flags.insert(DISPSTATFlags::HBLANK);
        }
        if self.vcount < 160 { // Visible
            self.dispstat.flags.remove(DISPSTATFlags::VBLANK);
        } else { // VBlank
            self.dispstat.flags.insert(DISPSTATFlags::VBLANK);
        }

        self.dot += 1;
        if self.dot == 308 {
            self.dot = 0;
            self.vcount = (self.vcount + 1) % 228;
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
