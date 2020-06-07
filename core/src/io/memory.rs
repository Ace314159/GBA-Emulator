use super::IO;

impl IO {
    pub(super) fn read_rom(&self, addr: u32) -> u8 {
        let addr = addr as usize - 0x08000000;
        if addr < self.rom.len() { self.rom[addr] }
        else { warn!("Returning Invalid ROM Read at 0x{:08X}", addr); 0 }
    }
}
