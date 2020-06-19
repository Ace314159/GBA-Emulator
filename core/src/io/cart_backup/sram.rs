use super::CartBackup;

pub struct SRAM {
    mem: [u8; 0x8000]
}

impl SRAM {
    pub fn new() -> SRAM {
        SRAM {
            mem: [0; 0x8000],
        }
    }
}

impl CartBackup for SRAM {
    fn read(&self, addr: u32) -> u8 {
        if addr < 0x8000 { self.mem[addr as usize] } else { 0 }
    }

    fn write(&mut self, addr: u32, value: u8) {
        if addr < 0x8000 { self.mem[addr as usize] = value }
    }
}