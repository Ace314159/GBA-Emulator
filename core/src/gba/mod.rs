use crate::cpu::CPU;
use crate::mmu::MMU;

pub struct GBA {
    cpu: CPU,
    mmu: MMU,
}

impl GBA {
    pub fn new(rom_file: &String) -> GBA {
        let bios = std::fs::read("gba_bios.bin").unwrap();
        let mmu = MMU::new(bios, std::fs::read(rom_file).unwrap());
        GBA {
            cpu: CPU::new(&mmu),
            mmu,
        }
    }

    pub fn emulate(&mut self) {
        self.cpu.emulate_instr(&mut self.mmu);
    }
}
