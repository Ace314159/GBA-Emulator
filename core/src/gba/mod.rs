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
            cpu: CPU::no_bios(&mut io),
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
