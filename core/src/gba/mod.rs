use crate::cpu::CPU;
use crate::io::IO;
pub use crate::io::keypad::KEYINPUT;

pub struct GBA {
    cpu: CPU,
    io: IO,
}

impl GBA {
    pub fn new(rom_file: String) -> GBA {
        let bios = std::fs::read("gba_bios.bin").unwrap();
        let mut io = IO::new(bios, std::fs::read(rom_file).unwrap());
        GBA {
            cpu: CPU::new(false, &mut io),
            io,
        }
    }

    pub fn emulate(&mut self) {
        self.io.run_dmas();
        self.cpu.handle_irq(&mut self.io);
        self.cpu.emulate_instr(&mut self.io);
    }

    pub fn needs_to_render(&mut self) -> bool {
        self.io.needs_to_render()
    }

    pub fn get_pixels(&self) -> &Vec<u16> {
        self.io.get_pixels()
    }

    pub fn render_map(&self, bg_i: usize) -> (Vec<u16>, usize, usize) {
        self.io.render_map(bg_i)
    }

    pub fn render_tiles(&self, palette: usize, block: usize, bpp8: bool) -> (Vec<u16>, usize, usize) {
        self.io.render_tiles(palette, block, bpp8)
    }

    pub fn render_palettes(&self) -> (Vec<u16>, usize, usize) {
        self.io.render_palettes()
    }

    pub fn peek_mem(&self, region: VisibleMemoryRegion, addr: usize) -> u8 {
        self.io.peek_mem(region, addr as u32)
    }

    pub fn press_key(&mut self, key: KEYINPUT) {
        self.io.press_key(key);
    }

    pub fn release_key(&mut self, key: KEYINPUT) {
        self.io.release_key(key);
    }
}

pub const WIDTH: usize = 240;
pub const HEIGHT: usize = 160;
pub const SCALE: usize = 2;

#[derive(Clone, Copy)]
pub enum VisibleMemoryRegion {
    BIOS = 0,
    EWRAM = 1,
    IWRAM = 2,
    IO = 3,
    Palette = 4,
    VRAM = 5,
    OAM = 6,
    PakROM = 7,
    CartRAM = 8,
}

impl VisibleMemoryRegion {
    pub fn from_index(index: usize) -> Self {
        use VisibleMemoryRegion::*;
        match index {
            0 => BIOS,
            1 => EWRAM,
            2 => IWRAM,
            3 => IO,
            4 => Palette,
            5 => VRAM,
            6 => OAM,
            7 => PakROM,
            8 => CartRAM,
            _ => unreachable!(),
        }
    }

    pub fn get_name(&self) -> String {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => "BIOS",
            EWRAM => "EWRAM",
            IWRAM => "IWRAM",
            IO => "IO",
            Palette => "Palette",
            VRAM => "VRAM",
            OAM => "OAM",
            PakROM => "Pak ROM",
            CartRAM => "Cart RAM",
        }.to_string()
    }

    pub fn get_size(&self) -> usize {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => 0x4000,
            EWRAM => 0x4_0000,
            IWRAM => 0x8000,
            IO => 0x400,
            Palette => 0x400,
            VRAM => 0x1_8000,
            OAM => 0x400,
            PakROM => 0x0600_0000,
            CartRAM => 0x1_0000,
        }
    }

    pub fn get_start_addr(&self) -> u32 {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => 0x0000_0000,
            EWRAM => 0x0200_0000,
            IWRAM => 0x0300_0000,
            IO => 0x0400_0000,
            Palette => 0x0500_0000,
            VRAM => 0x0600_0000,
            OAM => 0x0700_0000,
            PakROM => 0x0800_0000,
            CartRAM => 0x0E00_0000,
        }
    }
}
