use crate::cpu::CPU;
use crate::mmu::MMU;

pub struct GBA {
    cpu: CPU,
    mmu: MMU,
}

impl GBA {
    pub fn new(rom_file: &String) -> GBA {
        let bios = std::fs::read("gba_bios.bin").unwrap();
        let mut mmu = MMU::new(bios, std::fs::read(rom_file).unwrap());
        GBA {
            cpu: CPU::no_bios(&mut mmu),
            mmu,
        }
    }

    pub fn emulate(&mut self) {
        self.cpu.emulate_instr(&mut self.mmu);
    }

    pub fn needs_to_render(&mut self) -> bool {
        self.mmu.needs_to_render()
    }

    pub fn get_pixels(&self) -> &[u16; Screen::WIDTH * Screen::HEIGHT] {
        self.mmu.get_pixels()
    }
}

pub trait Screen {
    fn set_size(&mut self, width: i32, height: i32);
    fn render(&mut self, pixels: &[u16; Screen::WIDTH * Screen::HEIGHT]);
}

impl dyn Screen {
    pub const WIDTH: usize = 240;
    pub const HEIGHT: usize = 160;
    pub const SCALE: usize = 2;
}
