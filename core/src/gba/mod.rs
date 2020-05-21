use crate::cpu::CPU;
use crate::mmu::MMU;
pub use crate::mmu::keypad::KEYINPUT;

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

    pub fn get_pixels(&self) -> &[u16; Display::WIDTH * Display::HEIGHT] {
        self.mmu.get_pixels()
    }

    pub fn press_key(&mut self, key: KEYINPUT) {
        self.mmu.press_key(key);
    }

    pub fn release_key(&mut self, key: KEYINPUT) {
        self.mmu.release_key(key);
    }
}

pub trait Display {
    fn should_close(&self) -> bool;
    fn render(&mut self, pixels: &mut GBA);
}

impl dyn Display {
    pub const WIDTH: usize = 240;
    pub const HEIGHT: usize = 160;
    pub const SCALE: usize = 2;
}
