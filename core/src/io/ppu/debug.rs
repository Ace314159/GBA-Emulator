use super::{BGMode, PPU};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub struct DebugSpecification {
    pub map_enable: bool,
    pub tiles_enable: bool,
    pub palettes_enable: bool,

    pub map_spec: MapSpecification,
    pub tiles_spec: TilesSpecification,
}

pub type DebugWindows = VecDeque<(Vec<u16>, usize, usize)>;

#[derive(Clone, Copy)]
pub struct MapSpecification {
    pub bg_i: usize,
}

#[derive(Clone, Copy)]
pub struct TilesSpecification {
    pub palette: i32,
    pub block: usize,
    pub bpp8: bool
}

impl PPU {
    pub fn create_debug_windows(&self) -> DebugWindows {
        let spec_lock = self.debug_spec.lock().unwrap();
        let spec = spec_lock.clone();
        drop(spec_lock);
        
        let mut debug_windows = VecDeque::with_capacity(3);
        // TODO: Order shouldn't be arbritrary
        if spec.map_enable { debug_windows.push_back(self.render_map(&spec.map_spec)) }
        if spec.tiles_enable { debug_windows.push_back(self.render_tiles(&spec.tiles_spec)) }
        if spec.palettes_enable { debug_windows.push_back(self.render_palettes()) }

        debug_windows
    }

    fn render_map(&self, spec: &MapSpecification) -> (Vec<u16>, usize, usize) {
        let bg_i = spec.bg_i;

        let bgcnt = self.bgcnts[bg_i];
        let affine = self.dispcnt.mode == BGMode::Mode2 ||
                            self.dispcnt.mode == BGMode::Mode1 && bg_i == 2;
        let (width, height) = match bgcnt.screen_size {
            0 => [(256, 256), (128, 128)][affine as usize],
            1 => [(512, 256), (256, 256)][affine as usize],
            2 => [(256, 512), (512, 512)][affine as usize],
            3 => [(512, 512), (1024, 1024)][affine as usize],
            _ => unreachable!(),
        };
        let mut pixels = vec![0u16; width * height];
        let tile_start_addr = bgcnt.tile_block as usize * 0x4000;
        let map_start_addr = bgcnt.map_block as usize * 0x800;
        let bit_depth = if bgcnt.bpp8 { 8 } else { 4 }; // Also bytes per row of tile

        for y in 0..height {
            for x in 0..width {
                if affine {
                    // Get Screen Entry
                    let map_x = x / 8;
                    let map_y = y / 8;
                    let addr = map_start_addr + map_y * width / 8 + map_x;
                    let tile_num = self.vram[addr] as usize;
                    
                    // Convert from tile to pixels
                    let (_, color_num) = self.get_color_from_tile(tile_start_addr, tile_num,
                        false, false, 8, x % 8, y % 8, 0);
                    if color_num == 0 { continue }
                    pixels[y * width + x] = self.bg_palettes[color_num] | 0x8000;
                } else {
                    // Get Screen Entry
                    let map_x = x / 8;
                    let map_y = y / 8;
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
                        _ => unreachable!(),
                    };
                    let addr = map_start_addr + map_y * 32 * 2 + map_x * 2;
                    let screen_entry = u16::from_le_bytes([self.vram[addr], self.vram[addr + 1]]) as usize;
                    let tile_num = screen_entry & 0x3FF;
                    let flip_x = (screen_entry >> 10) & 0x1 != 0;
                    let flip_y = (screen_entry >> 11) & 0x1 != 0;
                    let palette_num = (screen_entry >> 12) & 0xF;
                    
                    // Convert from tile to pixels
                    let (palette_num, color_num) = self.get_color_from_tile(tile_start_addr,
                        tile_num, flip_x, flip_y, bit_depth, x % 8, y % 8, palette_num);
                    if color_num == 0 { continue }
                    pixels[y * width + x] = self.bg_palettes[palette_num * 16 + color_num] | 0x8000;
                }
            }
        }
        (pixels, width, height)
    }

    fn render_tiles(&self, spec: &TilesSpecification) -> (Vec<u16>, usize, usize) {
        let (palette, block, bpp8) = (spec.palette as usize, spec.block, spec.bpp8);

        let tile_start_addr = if block > 3 { 0x10000 } else { block * 0x4000 };
        let tiles_size = if bpp8 { 16 } else { 32 };
        let size = tiles_size * 8;
        let mut pixels = vec![0; size * size];
        let bit_depth = if bpp8 { 8 } else { 4 };
        for tile_y in 0..tiles_size {
            for tile_x in 0..tiles_size {
                let start_i = (tile_y * size + tile_x) * 8;
                for y in 0..8 {
                    for x in 0..8 {
                        let (palette_num, color_num) = self.get_color_from_tile(tile_start_addr,
                            tile_y * tiles_size + tile_x, false, false,
                            bit_depth, x, y, palette);
                        pixels[start_i + y * size + x] = self.bg_palettes[palette_num * 16 + color_num] | 0x8000;
                    }
                }
            }
        }
        (pixels, size, size)
    }

    fn render_palettes(&self) -> (Vec<u16>, usize, usize) {
        let palettes_size = 16;
        let size = palettes_size * 8;
        let mut pixels = vec![0; size * size];
        for palette_y in 0..palettes_size {
            for palette_x in 0..palettes_size {
                let palette_num = palette_y * palettes_size + palette_x;
                let start_i = (palette_y * size + palette_x) * 8;
                for y in 0..8 {
                    for x in 0..8 {
                        pixels[start_i + y * size + x] = self.bg_palettes[palette_num] | 0x8000;
                    }
                }
            }
        }
        (pixels, size, size)
    }
}

impl DebugSpecification {
    pub fn new() -> DebugSpecification {
        DebugSpecification {
            map_enable: false,
            tiles_enable: false,
            palettes_enable: false,

            map_spec: MapSpecification::new(),
            tiles_spec: TilesSpecification::new(),
        }
    }
}

impl MapSpecification {
    pub fn new() -> MapSpecification {
        MapSpecification {
            bg_i: 0,
        }
    }
}

impl TilesSpecification {
    pub fn new() -> TilesSpecification {
        TilesSpecification {
            palette: 0,
            block: 0,
            bpp8: false,
        }
    }
}
