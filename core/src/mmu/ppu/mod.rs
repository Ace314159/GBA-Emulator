mod registers;
mod layers;

use crate::gba::Display;
use super::MemoryHandler;
use super::IORegister;

use registers::*;
use layers::BG;

pub struct PPU {
    // Registers
    dispcnt: DISPCNT,
    green_swap: bool,
    dispstat: DISPSTAT,
    vcount: u8,
    // Layers1
    bg0: BG<BG01CNT>,
    bg1: BG<BG01CNT>,
    bg2: BG<BG23CNT>,
    bg3: BG<BG23CNT>,

    // Palettes
    bg_colors: [u16; 0x100],
    obj_colors: [u16; 0x100],
    // VRAM
    vram: [u8; 0x18000],

    // Important Rendering Variables
    dot: u16,
    pub pixels: [u16; Display::WIDTH * Display::HEIGHT],
    pub needs_to_render: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            // Registers
            dispcnt: DISPCNT::new(),
            green_swap: false,
            dispstat: DISPSTAT::new(),
            vcount: 0, 
            // Layer
            bg0: BG::new(),
            bg1: BG::new(),
            bg2: BG::new(),
            bg3: BG::new(),

            // Palettes
            bg_colors: [0; 0x100],
            obj_colors: [0; 0x100],
            // VRAM
            vram: [0; 0x18000],

            // Important Rendering Variables
            dot: 0,
            pixels: [0; Display::WIDTH * Display::HEIGHT],
            needs_to_render: false,
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

        if self.vcount == 160 && self.dot == 0 {
            match self.dispcnt.mode {
                BGMode::Mode0 => {}, // Do nothing temporarily to avoid crash
                BGMode::Mode4 => {
                    let start_addr = if self.dispcnt.flags.contains(DISPCNTFlags::DISPLAY_FRAME_SELECT) {
                        0xA000usize
                    } else { 0usize };
                    for i in 0..Display::WIDTH * Display::HEIGHT {
                        self.pixels[i] = self.bg_colors[self.vram[start_addr + i] as usize];
                    }
                },
                _ => unimplemented!("BG Mode {} not implemented", self.dispcnt.mode as u32),
            }
            self.needs_to_render = true;
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
            0x000 => self.dispcnt.read_low(),
            0x001 => self.dispcnt.read_high(),
            0x002 => self.green_swap as u8,
            0x003 => 0, // Unused area of Green Swap
            0x004 => self.dispstat.read_low(),
            0x005 => self.dispstat.read_high(),
            0x006 => self.vcount as u8,
            0x007 => 0, // Unused area of VCOUNT
            0x008 => self.bg0.cnt.read_low(),
            0x009 => self.bg0.cnt.read_high(),
            0x00A => self.bg1.cnt.read_low(),
            0x00B => self.bg1.cnt.read_high(),
            0x00C => self.bg2.cnt.read_low(),
            0x00D => self.bg2.cnt.read_high(),
            0x00E => self.bg3.cnt.read_low(),
            0x00F => self.bg3.cnt.read_high(),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => self.dispcnt.write_low(value),
            0x001 => self.dispcnt.write_high(value),
            0x002 => self.green_swap = value & 0x1 != 0,
            0x003 => {},
            0x004 => self.dispstat.write_low(value),
            0x005 => self.dispstat.write_high(value),
            0x006 => {},
            0x007 => {},
            0x008 => self.bg0.cnt.write_low(value),
            0x009 => self.bg0.cnt.write_high(value),
            0x00A => self.bg1.cnt.write_low(value),
            0x00B => self.bg1.cnt.write_high(value),
            0x00C => self.bg2.cnt.write_low(value),
            0x00D => self.bg2.cnt.write_high(value),
            0x00E => self.bg3.cnt.write_low(value),
            0x00F => self.bg3.cnt.write_high(value),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
