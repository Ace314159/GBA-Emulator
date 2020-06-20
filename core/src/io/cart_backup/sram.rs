use std::path::PathBuf;

use super::CartBackup;

pub struct SRAM {
    mem: Vec<u8>,
    save_file: PathBuf,
    is_dirty: bool,
}

impl SRAM {
    const SIZE: usize = 0x8000;

    pub fn new(save_file: PathBuf) -> SRAM {
        SRAM {
            mem: CartBackup::get_initial_mem(&save_file, 0, SRAM::SIZE),
            save_file,
            is_dirty: false,
        }
    }
}

impl CartBackup for SRAM {
    fn read(&self, addr: u32) -> u8 {
        let addr = addr as usize;
        if addr < SRAM::SIZE { self.mem[addr] } else { 0 }
    }

    fn write(&mut self, addr: u32, value: u8) {
        let addr = addr as usize;
        if addr < SRAM::SIZE { self.is_dirty = true; self.mem[addr] = value }
    }

    fn init_eeprom(&mut self, _dma_count: u32) {}
    fn read_eeprom(&self, _addr: u32) -> u16 { unreachable!() }
    fn write_eeprom(&mut self, _addr: u32, _value: u16) { unreachable!() }
    fn is_dirty(&mut self) -> bool { let is_dirty = self.is_dirty; self.is_dirty = false; is_dirty }
    fn get_save_file(&self) -> &PathBuf { &self.save_file }
    fn get_mem(&self) -> &Vec<u8> { &self.mem }
    fn is_eeprom(&self) -> bool { false }
}