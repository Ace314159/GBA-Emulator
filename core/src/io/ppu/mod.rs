mod registers;

use crate::gba::Display;
use super::MemoryHandler;
use super::IORegister;

use registers::*;

pub struct PPU {
    // Registers
    dispcnt: DISPCNT,
    green_swap: bool,
    dispstat: DISPSTAT,
    vcount: u8,
    // Backgrounds
    bgcnts: [BGCNT; 4],
    hofs: [OFS; 4],
    vofs: [OFS; 4],
    pas: [RotationScalingParameter; 2],
    pbs: [RotationScalingParameter; 2],
    pcs: [RotationScalingParameter; 2],
    pds: [RotationScalingParameter; 2],
    bgxs: [ReferencePointCoord; 2],
    bgys: [ReferencePointCoord; 2],

    // Palettes
    bg_palettes: [u16; 0x100],
    obj_paletes: [u16; 0x100],
    // VRAM
    vram: [u8; 0x18000],
    oam: [u8; 0x400],

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
            // Backgrounds
            bgcnts: [BGCNT::new(); 4],
            hofs: [OFS::new(); 4],
            vofs: [OFS::new(); 4],
            pas: [RotationScalingParameter::new(); 2],
            pbs: [RotationScalingParameter::new(); 2],
            pcs: [RotationScalingParameter::new(); 2],
            pds: [RotationScalingParameter::new(); 2],
            bgxs: [ReferencePointCoord::new(); 2],
            bgys: [ReferencePointCoord::new(); 2],

            // Palettes
            bg_palettes: [0; 0x100],
            obj_paletes: [0; 0x100],
            // VRAM
            vram: [0; 0x18000],
            oam: [0; 0x400],

            // Important Rendering Variables
            dot: 0,
            pixels: [0; Display::WIDTH * Display::HEIGHT],
            needs_to_render: false,
        }
    }

    pub fn read_palette_ram(&self, addr: u32) -> u8 {
        let addr = (addr & 0x3FF) as usize;
        let palettes = if addr < 0x200 { &self.bg_palettes } else { &self.obj_paletes };
        let index = (addr & 0x1FF) / 2;
        if addr % 2 == 0 {
            (palettes[index] >> 0) as u8
        } else {
            (palettes[index] >> 8) as u8
        }
    }

    pub fn write_palette_ram(&mut self, addr: u32, value: u8) {
        let addr = (addr & 0x3FF) as usize;
        let palettes = if addr < 0x200 { &mut self.bg_palettes } else { &mut self.obj_paletes };
        let index = (addr & 0x1FF) / 2;
        if addr % 2 == 0 {
            palettes[index] = palettes[index] & !0x00FF | (value as u16) << 0;
        } else {
            palettes[index] = palettes[index] & !0xFF00 | (value as u16) << 8;
        }
    }

    pub fn read_vram(&self, addr: u32) -> u8 {
        self.vram[(addr - 0x06000000) as usize]
    }

    pub fn write_vram(&mut self, addr: u32, value: u8) {
        self.vram[(addr - 0x06000000) as usize] = value;
    }

    pub fn read_oam(&self, addr: u32) -> u8 {
        self.oam[(addr - 0x07000000) as usize]
    }

    pub fn write_oam(&mut self, addr: u32, value: u8) {
        self.oam[(addr - 0x07000000) as usize] = value;
    }

    pub fn emulate_dot(&mut self) {
        if self.dot < 240 { // Visible
            self.dispstat.remove(DISPSTATFlags::HBLANK);
        } else { // HBlank
            self.dispstat.insert(DISPSTATFlags::HBLANK);
        }
        if self.vcount < 160 { // Visible
            self.dispstat.remove(DISPSTATFlags::VBLANK);
        } else { // VBlank
            self.dispstat.insert(DISPSTATFlags::VBLANK);
        }

        if self.vcount == 160 && self.dot == 0 {
            match self.dispcnt.mode {
                BGMode::Mode0 => {}, // Do nothing temporarily to avoid crash
                BGMode::Mode4 => {
                    let start_addr = if self.dispcnt.contains(DISPCNTFlags::DISPLAY_FRAME_SELECT) {
                        0xA000usize
                    } else { 0usize };
                    for i in 0..Display::WIDTH * Display::HEIGHT {
                        self.pixels[i] = self.bg_palettes[self.vram[start_addr + i] as usize];
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
            0x008 => self.bgcnts[0].read(0),
            0x009 => self.bgcnts[0].read(1),
            0x00A => self.bgcnts[1].read(0),
            0x00B => self.bgcnts[1].read(1),
            0x00C => self.bgcnts[2].read(0),
            0x00D => self.bgcnts[2].read(1),
            0x00E => self.bgcnts[3].read(0),
            0x00F => self.bgcnts[3].read(1),
            0x010 => self.hofs[0].read(0),
            0x011 => self.hofs[0].read(1),
            0x012 => self.vofs[0].read(0),
            0x013 => self.vofs[0].read(1),
            0x014 => self.hofs[1].read(1),
            0x015 => self.hofs[1].read(0),
            0x016 => self.vofs[1].read(0),
            0x017 => self.vofs[1].read(1),
            0x018 => self.hofs[2].read(0),
            0x019 => self.hofs[2].read(1),
            0x01A => self.vofs[2].read(0),
            0x01B => self.vofs[2].read(1),
            0x01C => self.hofs[3].read(0),
            0x01D => self.hofs[3].read(1),
            0x01E => self.vofs[3].read(0),
            0x01F => self.vofs[3].read(1),
            0x020 => self.pas[0].read(0),
            0x021 => self.pas[0].read(1),
            0x022 => self.pbs[0].read(0),
            0x023 => self.pbs[0].read(1),
            0x024 => self.pcs[0].read(0),
            0x025 => self.pcs[0].read(1),
            0x026 => self.pds[0].read(0),
            0x027 => self.pds[0].read(1),
            0x028 => self.bgxs[0].read(0),
            0x029 => self.bgxs[0].read(1),
            0x02A => self.bgxs[0].read(2),
            0x02B => self.bgxs[0].read(3),
            0x02C => self.bgys[0].read(0),
            0x02D => self.bgys[0].read(1),
            0x02E => self.bgys[0].read(2),
            0x02F => self.bgys[0].read(3),
            0x030 => self.pas[1].read(0),
            0x031 => self.pas[1].read(1),
            0x032 => self.pbs[1].read(0),
            0x033 => self.pbs[1].read(1),
            0x034 => self.pcs[1].read(0),
            0x035 => self.pcs[1].read(1),
            0x036 => self.pds[1].read(0),
            0x037 => self.pds[1].read(1),
            0x038 => self.bgxs[1].read(0),
            0x039 => self.bgxs[1].read(1),
            0x03A => self.bgxs[1].read(2),
            0x03B => self.bgxs[1].read(3),
            0x03C => self.bgys[1].read(0),
            0x03D => self.bgys[1].read(1),
            0x03E => self.bgys[1].read(2),
            0x03F => self.bgys[1].read(3),
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
            0x008 => self.bgcnts[0].write(0, value),
            0x009 => self.bgcnts[0].write(1, value),
            0x00A => self.bgcnts[1].write(0, value),
            0x00B => self.bgcnts[1].write(1, value),
            0x00C => self.bgcnts[2].write(0, value),
            0x00D => self.bgcnts[2].write(1, value),
            0x00E => self.bgcnts[3].write(0, value),
            0x00F => self.bgcnts[3].write(1, value),
            0x010 => self.hofs[0].write(0, value),
            0x011 => self.hofs[0].write(1, value),
            0x012 => self.vofs[0].write(0, value),
            0x013 => self.vofs[0].write(1, value),
            0x014 => self.hofs[1].write(1, value),
            0x015 => self.hofs[1].write(0, value),
            0x016 => self.vofs[1].write(0, value),
            0x017 => self.vofs[1].write(1, value),
            0x018 => self.hofs[2].write(0, value),
            0x019 => self.hofs[2].write(1, value),
            0x01A => self.vofs[2].write(0, value),
            0x01B => self.vofs[2].write(1, value),
            0x01C => self.hofs[3].write(0, value),
            0x01D => self.hofs[3].write(1, value),
            0x01E => self.vofs[3].write(0, value),
            0x01F => self.vofs[3].write(1, value),
            0x020 => self.pas[0].write(0, value),
            0x021 => self.pas[0].write(1, value),
            0x022 => self.pbs[0].write(0, value),
            0x023 => self.pbs[0].write(1, value),
            0x024 => self.pcs[0].write(0, value),
            0x025 => self.pcs[0].write(1, value),
            0x026 => self.pds[0].write(0, value),
            0x027 => self.pds[0].write(1, value),
            0x028 => self.bgxs[0].write(0, value),
            0x029 => self.bgxs[0].write(1, value),
            0x02A => self.bgxs[0].write(2, value),
            0x02B => self.bgxs[0].write(3, value),
            0x02C => self.bgys[0].write(0, value),
            0x02D => self.bgys[0].write(1, value),
            0x02E => self.bgys[0].write(2, value),
            0x02F => self.bgys[0].write(3, value),
            0x030 => self.pas[1].write(0, value),
            0x031 => self.pas[1].write(1, value),
            0x032 => self.pbs[1].write(0, value),
            0x033 => self.pbs[1].write(1, value),
            0x034 => self.pcs[1].write(0, value),
            0x035 => self.pcs[1].write(1, value),
            0x036 => self.pds[1].write(0, value),
            0x037 => self.pds[1].write(1, value),
            0x038 => self.bgxs[1].write(0, value),
            0x039 => self.bgxs[1].write(1, value),
            0x03A => self.bgxs[1].write(2, value),
            0x03B => self.bgxs[1].write(3, value),
            0x03C => self.bgys[1].write(0, value),
            0x03D => self.bgys[1].write(1, value),
            0x03E => self.bgys[1].write(2, value),
            0x03F => self.bgys[1].write(3, value),
            _ => unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
