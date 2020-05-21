mod registers;
mod layers;

use crate::gba::Display;
use super::MemoryHandler;
use super::IORegister;

use registers::*;
use layers::{BG01, BG23};

pub struct PPU {
    // Registers
    dispcnt: DISPCNT,
    green_swap: bool,
    dispstat: DISPSTAT,
    vcount: u8,
    // Layers1
    bg0: BG01,
    bg1: BG01,
    bg2: BG23,
    bg3: BG23,

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
            bg0: BG01::new(),
            bg1: BG01::new(),
            bg2: BG23::new(),
            bg3: BG23::new(),

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
            0x000 => self.dispcnt.read(0),
            0x001 => self.dispcnt.read(1),
            0x002 => self.green_swap as u8,
            0x003 => 0, // Unused area of Green Swap
            0x004 => self.dispstat.read(0),
            0x005 => self.dispstat.read(1),
            0x006 => self.vcount as u8,
            0x007 => 0, // Unused area of VCOUNT
            0x008 => self.bg0.cnt.read(0),
            0x009 => self.bg0.cnt.read(1),
            0x00A => self.bg1.cnt.read(0),
            0x00B => self.bg1.cnt.read(1),
            0x00C => self.bg2.cnt.read(0),
            0x00D => self.bg2.cnt.read(1),
            0x00E => self.bg3.cnt.read(0),
            0x00F => self.bg3.cnt.read(1),
            0x010 => self.bg0.hofs.read(0),
            0x011 => self.bg0.hofs.read(1),
            0x012 => self.bg0.vofs.read(0),
            0x013 => self.bg0.vofs.read(1),
            0x014 => self.bg1.hofs.read(1),
            0x015 => self.bg1.hofs.read(0),
            0x016 => self.bg1.vofs.read(0),
            0x017 => self.bg1.vofs.read(1),
            0x018 => self.bg2.hofs.read(0),
            0x019 => self.bg2.hofs.read(1),
            0x01A => self.bg2.vofs.read(0),
            0x01B => self.bg2.vofs.read(1),
            0x01C => self.bg3.hofs.read(0),
            0x01D => self.bg3.hofs.read(1),
            0x01E => self.bg3.vofs.read(0),
            0x01F => self.bg3.vofs.read(1),
            0x020 => self.bg2.pa.read(0),
            0x021 => self.bg2.pa.read(1),
            0x022 => self.bg2.pb.read(0),
            0x023 => self.bg2.pb.read(1),
            0x024 => self.bg2.pc.read(0),
            0x025 => self.bg2.pc.read(1),
            0x026 => self.bg2.pd.read(0),
            0x027 => self.bg2.pd.read(1),
            0x028 => self.bg2.ref_point_x.read(0, ),
            0x029 => self.bg2.ref_point_x.read(1, ),
            0x02A => self.bg2.ref_point_x.read(2, ),
            0x02B => self.bg2.ref_point_x.read(3, ),
            0x02C => self.bg2.ref_point_x.read(0, ),
            0x02D => self.bg2.ref_point_x.read(1, ),
            0x02E => self.bg2.ref_point_x.read(2, ),
            0x02F => self.bg2.ref_point_x.read(3, ),
            0x030 => self.bg3.pa.read(0),
            0x031 => self.bg3.pa.read(1),
            0x032 => self.bg3.pb.read(0),
            0x033 => self.bg3.pb.read(1),
            0x034 => self.bg3.pc.read(0),
            0x035 => self.bg3.pc.read(1),
            0x036 => self.bg3.pd.read(0),
            0x037 => self.bg3.pd.read(1),
            0x038 => self.bg3.ref_point_x.read(0, ),
            0x039 => self.bg3.ref_point_x.read(1, ),
            0x03A => self.bg3.ref_point_x.read(2, ),
            0x03B => self.bg3.ref_point_x.read(3, ),
            0x03C => self.bg3.ref_point_x.read(0, ),
            0x03D => self.bg3.ref_point_x.read(1, ),
            0x03E => self.bg3.ref_point_x.read(2, ),
            0x03F => self.bg3.ref_point_x.read(3, ),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x000 => self.dispcnt.write(0, value),
            0x001 => self.dispcnt.write(1, value),
            0x002 => self.green_swap = value & 0x1 != 0,
            0x003 => {},
            0x004 => self.dispstat.write(0, value),
            0x005 => self.dispstat.write(1, value),
            0x006 => {},
            0x007 => {},
            0x008 => self.bg0.cnt.write(0, value),
            0x009 => self.bg0.cnt.write(1, value),
            0x00A => self.bg1.cnt.write(0, value),
            0x00B => self.bg1.cnt.write(1, value),
            0x00C => self.bg2.cnt.write(0, value),
            0x00D => self.bg2.cnt.write(1, value),
            0x00E => self.bg3.cnt.write(0, value),
            0x00F => self.bg3.cnt.write(1, value),
            0x010 => self.bg0.hofs.write(0, value),
            0x011 => self.bg0.hofs.write(1, value),
            0x012 => self.bg0.vofs.write(0, value),
            0x013 => self.bg0.vofs.write(1, value),
            0x014 => self.bg1.hofs.write(1, value),
            0x015 => self.bg1.hofs.write(0, value),
            0x016 => self.bg1.vofs.write(0, value),
            0x017 => self.bg1.vofs.write(1, value),
            0x018 => self.bg2.hofs.write(0, value),
            0x019 => self.bg2.hofs.write(1, value),
            0x01A => self.bg2.vofs.write(0, value),
            0x01B => self.bg2.vofs.write(1, value),
            0x01C => self.bg3.hofs.write(0, value),
            0x01D => self.bg3.hofs.write(1, value),
            0x01E => self.bg3.vofs.write(0, value),
            0x01F => self.bg3.vofs.write(1, value),
            0x020 => self.bg2.pa.write(0, value),
            0x021 => self.bg2.pa.write(1, value),
            0x022 => self.bg2.pb.write(0, value),
            0x023 => self.bg2.pb.write(1, value),
            0x024 => self.bg2.pc.write(0, value),
            0x025 => self.bg2.pc.write(1, value),
            0x026 => self.bg2.pd.write(0, value),
            0x027 => self.bg2.pd.write(1, value),
            0x028 => self.bg2.ref_point_x.write(0, value),
            0x029 => self.bg2.ref_point_x.write(1, value),
            0x02A => self.bg2.ref_point_x.write(2, value),
            0x02B => self.bg2.ref_point_x.write(3, value),
            0x02C => self.bg2.ref_point_x.write(0, value),
            0x02D => self.bg2.ref_point_x.write(1, value),
            0x02E => self.bg2.ref_point_x.write(2, value),
            0x02F => self.bg2.ref_point_x.write(3, value),
            0x030 => self.bg3.pa.write(0, value),
            0x031 => self.bg3.pa.write(1, value),
            0x032 => self.bg3.pb.write(0, value),
            0x033 => self.bg3.pb.write(1, value),
            0x034 => self.bg3.pc.write(0, value),
            0x035 => self.bg3.pc.write(1, value),
            0x036 => self.bg3.pd.write(0, value),
            0x037 => self.bg3.pd.write(1, value),
            0x038 => self.bg3.ref_point_x.write(0, value),
            0x039 => self.bg3.ref_point_x.write(1, value),
            0x03A => self.bg3.ref_point_x.write(2, value),
            0x03B => self.bg3.ref_point_x.write(3, value),
            0x03C => self.bg3.ref_point_x.write(0, value),
            0x03D => self.bg3.ref_point_x.write(1, value),
            0x03E => self.bg3.ref_point_x.write(2, value),
            0x03F => self.bg3.ref_point_x.write(3, value),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
