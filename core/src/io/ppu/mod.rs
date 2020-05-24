mod registers;

use crate::gba::Display;
use super::MemoryHandler;
use super::IORegister;
use super::interrupt_controller::InterruptRequest;

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
    // Windows
    winhs: [WindowDimensions; 2],
    winvs: [WindowDimensions; 2],
    win_0_cnt: WindowControl,
    win_1_cnt: WindowControl,
    win_out_cnt: WindowControl,
    win_obj_cnt: WindowControl,

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

    p: bool,
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
            winhs: [WindowDimensions::new(); 2],
            winvs: [WindowDimensions::new(); 2],
            win_0_cnt: WindowControl::new(),
            win_1_cnt: WindowControl::new(),
            win_out_cnt: WindowControl::new(),
            win_obj_cnt: WindowControl::new(),

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

            p: false,
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

    pub fn emulate_dot(&mut self) -> InterruptRequest {
        let mut interrupts = InterruptRequest::empty();
        if self.dot < 240 { // Visible
            self.dispstat.remove(DISPSTATFlags::HBLANK);
        } else { // HBlank
            interrupts.insert(InterruptRequest::HBLANK);
            self.dispstat.insert(DISPSTATFlags::HBLANK);
            if self.vcount < 160 && self.dot == 241 { self.render_line() }
        }
        if self.vcount < 160 && self.vcount != 227 { // Visible
            self.dispstat.remove(DISPSTATFlags::VBLANK);
        } else { // VBlank
            interrupts.insert(InterruptRequest::VBLANK);
            self.dispstat.insert(DISPSTATFlags::VBLANK);
        }

        if self.vcount == 160 && self.dot == 0 { self.needs_to_render = true }

        self.dot += 1;
        if self.dot == 308 {
            self.dot = 0;
            self.vcount = (self.vcount + 1) % 228;
            if self.vcount == self.dispstat.vcount_setting {
                interrupts.insert(InterruptRequest::VCOUNTER_MATCH);
            }
        }
        interrupts
    }

    const OBJ_SIZES: [[(i16, u16); 3]; 4] = [
        [(8, 8), (16, 8), (8, 16)],
        [(16, 16), (32, 8), (8, 32)],
        [(32, 32), (32, 16), (16, 32)],
        [(64, 64), (64, 32), (32, 64)],
    ];

    fn render_line(&mut self) {
        let mut bg_priorities = [4u16; Display::WIDTH];
        use BGMode::*;
        let start_index = self.vcount as usize * Display::WIDTH;
        match self.dispcnt.mode {
            Mode0 => {
                let mut bgs: Vec<(usize, u8)> = Vec::new();
                if self.dispcnt.contains(DISPCNTFlags::DISPLAY_BG3) { bgs.push((3, self.bgcnts[3].priority)) }
                if self.dispcnt.contains(DISPCNTFlags::DISPLAY_BG2) { bgs.push((2, self.bgcnts[2].priority)) }
                if self.dispcnt.contains(DISPCNTFlags::DISPLAY_BG1) { bgs.push((1, self.bgcnts[1].priority)) }
                if self.dispcnt.contains(DISPCNTFlags::DISPLAY_BG0) { bgs.push((0, self.bgcnts[0].priority)) }
                bgs.sort_by_key(|a| a.1);

                let backdrop_color = self.bg_palettes[0];
                self.pixels[start_index..start_index + Display::WIDTH]
                .iter_mut().for_each(|x| *x = backdrop_color);
                bgs.iter().for_each(|(bg_i, _)| self.render_text_line(*bg_i, &mut bg_priorities));
            }
            Mode3 => {
                for i in start_index..start_index + Display::WIDTH {
                    self.pixels[i] = u16::from_le_bytes([self.vram[i * 2], self.vram[i * 2 + 1]]);
                }
            },
            Mode4 => {
                let start_addr = if self.dispcnt.contains(DISPCNTFlags::DISPLAY_FRAME_SELECT) {
                    0xA000usize
                } else { 0usize };
                for i in start_index..start_index + Display::WIDTH {
                    self.pixels[i] = self.bg_palettes[self.vram[start_addr + i] as usize];
                }
            },
            Mode5 => {
                let mut addr = if self.dispcnt.contains(DISPCNTFlags::DISPLAY_FRAME_SELECT) {
                    0xA000usize
                } else { 0usize };
                let dot_y = self.vcount as usize;
                addr += dot_y * 160 * 2;
                for dot_x in 0..Display::WIDTH {
                    self.pixels[dot_y * Display::WIDTH + dot_x] = if dot_x >= 160 || dot_y >= 128 {
                        self.bg_palettes[0]
                    } else {
                        let pixel = u16::from_le_bytes([self.vram[addr], self.vram[addr + 1]]);
                        addr += 2;
                        pixel
                    }
                }
            }
            _ => unimplemented!("BG Mode {} not implemented", self.dispcnt.mode as u32),
        }

        if self.dispcnt.contains(DISPCNTFlags::DISPLAY_OBJ) { self.render_oam_line(&bg_priorities)}
    }

    fn render_oam_line(&mut self, bg_priorities: &[u16; Display::WIDTH]) {
        let mut oam_parsed = [[0u16; 3]; 0x80];
        (0..self.oam.len()).filter(|i| i % 2 == 0 && i & 0x7 != 6)
        .for_each(|i| oam_parsed[i / 8][i / 2 % 4] = u16::from_le_bytes([self.oam[i], self.oam[i + 1]]));
        oam_parsed.sort_by_key(|obj| (obj[0] >> 14 & 0x3, obj[1] >> 14 & 0x3));
        let objs = oam_parsed.iter().filter(|obj| {
            let obj_shape = (obj[0] >> 14 & 0x3) as usize;
            let obj_size = (obj[1] >> 14 & 0x3) as usize;
            let (_, obj_height) = PPU::OBJ_SIZES[obj_size][obj_shape];
            let rot_scaling = obj[0] >> 8 & 0x1 != 0;
            let double_size_disable = obj[0] >> 9 & 0x1 != 0;
            if !rot_scaling && double_size_disable { return false }
            let obj_height = if double_size_disable { obj_height * 2 } else { obj_height };
            
            let obj_y = (obj[0] as u16) & 0xFF;
            let y_end = obj_y + obj_height;
            let y = self.vcount as u16 + if y_end > 256 { 256 } else { 0 };
            (obj_y..y_end).contains(&y)
        }).collect::<Vec<_>>();

        for dot_x in 0..Display::WIDTH {
            for obj in objs.iter() {
                let obj_shape = (obj[0] >> 14 & 0x3) as usize;
                let obj_size = (obj[1] >> 14 & 0x3) as usize;
                let double_size = obj[0] >> 9 & 0x1 != 0;
                let (obj_width, _) = PPU::OBJ_SIZES[obj_size][obj_shape];
                let obj_width = if double_size { obj_width * 2 } else { obj_width };
                let dot_x_signed = dot_x as i16;
                let obj_x = (obj[1] & 0x1FF) as u16;
                let obj_x = if obj_x & 0x100 != 0 { 0xFE00 | obj_x } else { obj_x } as i16;
                let obj_y = (obj[0] & 0xFF) as u16;
                if !(obj_x..obj_x + obj_width).contains(&dot_x_signed) { continue }

                let base_tile_num = (obj[2] & 0x3FF) as usize;
                let x_diff = dot_x_signed - obj_x;
                let y_diff = (self.vcount as u16).wrapping_sub(obj_y) & 0xFF;
                let tile_num = base_tile_num + if self.dispcnt.contains(DISPCNTFlags::OBJ_TILES1D) {
                    (y_diff as i16 / 8 * obj_width + x_diff) / 8
                } else { 0 } as usize; // TODO: Implement 2D Mapping
                let flip_x = obj[1] >> 12 & 0x1 != 0;
                let flip_y = obj[1] >> 13 & 0x1 != 0;
                let bit_depth = if obj[0] >> 13 & 0x1 != 0 { 8 } else { 4 };
                let tile_x = x_diff % 8;
                let tile_y = y_diff % 8;
                let palette_num = (obj[2] >> 12 & 0xF) as usize;
                let (palette_num, color_num) = self.get_color_from_tile(0x10000, tile_num,
                    flip_x, flip_y, bit_depth, tile_x as usize, tile_y as usize, palette_num);
                if color_num == 0 || bg_priorities[dot_x] < obj[2] >> 10 & 0x3 { continue }
                self.pixels[(self.vcount as usize) * Display::WIDTH + dot_x] = self.obj_paletes[palette_num * 16 + color_num];
            }
        }
    }

    fn render_text_line(&mut self, bg_i: usize, bg_priorities: &mut [u16; Display::WIDTH]) {
        let x_offset = self.hofs[bg_i].offset as usize;
        let y_offset = self.vofs[bg_i].offset as usize;
        let bgcnt = self.bgcnts[bg_i];
        let tile_start_addr = bgcnt.tile_block as usize * 0x4000;
        let map_start_addr = bgcnt.map_block as usize * 0x800;
        let bit_depth = if bgcnt.bpp8 { 8 } else { 4 }; // Also bytes per row of tile

        let dot_y = self.vcount as usize;
        for dot_x in 0..Display::WIDTH {
            let x = dot_x + x_offset;
            let y = dot_y + y_offset;
            // Get Screen Entry
            let mut map_x = x / 8;
            let mut map_y = y / 8;
            let map_start_addr = map_start_addr + match bgcnt.screen_size {
                0 => 0,
                1 => if (map_x / 32) % 2 == 1 { 0x800 } else { 0 },
                2 => if (map_y / 32) % 2 == 1 { 0x800 } else { 0 },
                3 => {
                    let x_overflowed = (map_x / 32) % 2 == 1;
                    let y_overflowed = (map_y / 32) % 2 == 1;
                    if x_overflowed && y_overflowed { 0x800 * 3 }
                    else if y_overflowed { 0x800 * 2 }
                    else if x_overflowed { 0x800 * 1 }
                    else { 0 }
                },
                _ => panic!("Invalid BG Size!"),
            };
            map_x %= 32;
            map_y %= 32;
            let addr = map_start_addr + map_y * 32 * 2 + map_x * 2;
            let screen_entry = u16::from_le_bytes([self.vram[addr], self.vram[addr + 1]]) as usize;
            let tile_num = screen_entry & 0x3FF;
            let flip_x = (screen_entry >> 10) & 0x1 != 0;
            let flip_y = (screen_entry >> 11) & 0x1 != 0;
            let palette_num = (screen_entry >> 12) & 0xF;
            
            // Convert from tile to pixels
            let (palette_num, color_num) = self.get_color_from_tile(tile_start_addr, tile_num, flip_x, flip_y,
                bit_depth, x % 8, y % 8, palette_num);
            if color_num == 0 { continue }
            bg_priorities[dot_x] = bgcnt.priority as u16;
            self.pixels[dot_y * Display::WIDTH + dot_x] = self.bg_palettes[palette_num * 16 + color_num];
        }
    }

    fn get_color_from_tile(&self, tile_start_addr: usize, tile_num: usize, flip_x: bool, flip_y: bool,
        bit_depth: usize, tile_x: usize, tile_y: usize, palette_num: usize) -> (usize, usize) {
        let addr = tile_start_addr + 8 * bit_depth * tile_num;
        let tile_x = if flip_x { 7 - tile_x } else { tile_x };
        let tile_y = if flip_y { 7 - tile_y } else { tile_y };
        let tile = self.vram[addr + tile_y * bit_depth + tile_x / (8 / bit_depth)] as usize;
        if bit_depth == 8 {
            (0, tile)
        } else {
            (palette_num, ((tile >> 4 * (tile_x % 2)) & 0xF))
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
            0x040 => self.winhs[0].read(0),
            0x041 => self.winhs[0].read(1),
            0x042 => self.winhs[1].read(0),
            0x043 => self.winhs[1].read(1),
            0x044 => self.winvs[0].read(0),
            0x045 => self.winvs[0].read(1),
            0x046 => self.winvs[1].read(0),
            0x047 => self.winvs[1].read(1),
            0x048 => self.win_0_cnt.read(0),
            0x049 => self.win_1_cnt.read(0),
            0x04A => self.win_out_cnt.read(0),
            0x04B => self.win_obj_cnt.read(0),
            _ => { if self.p { println!("Ignoring PPU Read at 0x{:08X}", addr) } 0 },
            // unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
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
            0x040 => self.winhs[0].write(0, value),
            0x041 => self.winhs[0].write(1, value),
            0x042 => self.winhs[1].write(0, value),
            0x043 => self.winhs[1].write(1, value),
            0x044 => self.winvs[0].write(0, value),
            0x045 => self.winvs[0].write(1, value),
            0x046 => self.winvs[1].write(0, value),
            0x047 => self.winvs[1].write(1, value),
            0x048 => self.win_0_cnt.write(0, value),
            0x049 => self.win_1_cnt.write(0, value),
            0x04A => self.win_out_cnt.write(0, value),
            0x04B => self.win_obj_cnt.write(0, value),
            _ => if self.p { println!("Ignoring PPU Write 0x{:08X} = {:02X}", addr, value) },
            //unimplemented!("PPU Handler for 0x{:08X} not implemented!", addr),
        }
    }
}
