mod memory;

use memory::ROM;
use memory::RAM;

pub struct MMU {
    bios: ROM,
    wram_32: RAM,
    _rom: ROM,
    clocks_ahead: u32,

    // Registers
    ime: bool,
    haltcnt: u16,
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            bios: ROM::new(bios),
            wram_32: RAM::new(0x03000000, 0x8000),
            _rom: ROM::new(rom),
            clocks_ahead: 0,

            // Registers
            ime: false,
            haltcnt: 0,
        }
    }
}

impl IMMU for MMU {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, _access_width: u32) {
        if cycle_type == Cycle::I { self.clocks_ahead += 1; return }
        self.clocks_ahead += match addr {
            0x00000000 ..= 0x00003FFF => 1, // BIOS ROM
            0x00004000 ..= 0x01FFFFFF => 1, // Unused Memory
            0x02040000 ..= 0x02FFFFFF => 1, // Unused Memory
            0x03000000 ..= 0x03007FFF => 1,
            0x03008000 ..= 0x03FFFFFF => 1, // Unused Memory
            0x04000000 ..= 0x040003FE => 1, // IO
            0x04000400 ..= 0x04FFFFFF => 1, // Unused Memory
            _ => unimplemented!("Clock Cycle for 0x{:08X} not implemented!", addr),
        };
    }
}

impl MemoryHandler for MMU {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.read8(addr),
            0x00004000 ..= 0x01FFFFFF => { 0 }, // Unused Memory
            0x02040000 ..= 0x02FFFFFF => { 0 }, // Unused Memory
            0x03000000 ..= 0x03007FFF => self.wram_32.read8(addr),
            0x03008000 ..= 0x03FFFFFF => { 0 }, // Unused Memory
            0x04000208 => self.ime as u8,
            0x04000300 => self.haltcnt as u8,
            0x04000301 => (self.haltcnt >> 8) as u8,
            0x04000400 ..= 0x04FFFFFF => { 0 }, // Unused Memory
            _ => unimplemented!("Memory Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.write8(addr, value),
            0x00004000 ..= 0x01FFFFFF => {}, // Unused Memory
            0x02040000 ..= 0x02FFFFFF => {}, // Unused Memory
            0x03000000 ..= 0x03007FFF => self.wram_32.write8(addr, value),
            0x03008000 ..= 0x03FFFFFF => {}, // Unused Memory
            0x04000208 => self.ime = value & 0x1 != 0,
            0x04000300 => self.haltcnt = (self.haltcnt & !0x00FF) | value as u16,
            0x04000301 => self.haltcnt = (self.haltcnt & !0xFF00) | (value as u16) << 8,
            0x04000400 ..= 0x04FFFFFF => {}, // Unused Memory
            _ => unimplemented!("Memory Handler for 0x{:08X} not implemented!", addr),
        }
    }
}

pub trait MemoryHandler {
    fn read8(&self, addr: u32) -> u8;
    fn write8(&mut self, addr: u32, value: u8);

    fn read16(&self, addr: u32) -> u16 {
        (self.read8(addr + 0) as u16) << 0 |
        (self.read8(addr + 1) as u16) << 8 
    }
    fn write16(&mut self, addr: u32, value: u16) {
        self.write8(addr + 0, (value >> 0) as u8);
        self.write8(addr + 1, (value >> 8) as u8);
    }

    fn read32(&self, addr: u32) -> u32 {
        (self.read8(addr + 0) as u32) << 0 |
        (self.read8(addr + 1) as u32) << 8 |
        (self.read8(addr + 2) as u32) << 16 |
        (self.read8(addr + 3) as u32) << 24
    }
    fn write32(&mut self, addr: u32, value: u32) {
        self.write8(addr + 0, (value >> 0) as u8);
        self.write8(addr + 1, (value >> 8) as u8);
        self.write8(addr + 2, (value >> 16) as u8);
        self.write8(addr + 3, (value >> 24) as u8);
    }
}

pub trait IMMU: MemoryHandler {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32);
}

#[derive(PartialEq)]
pub enum Cycle {
    N,
    S,
    I,
    // C - No coprocessor in GBA
}
