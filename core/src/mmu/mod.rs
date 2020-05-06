mod memory;

use memory::ROM;

pub struct MMU {
    bios: ROM,
    _rom: ROM,
    clocks_ahead: u32,
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            bios: ROM::new(bios),
            _rom: ROM::new(rom),
            clocks_ahead: 0,
        }
    }
}

impl IMMU for MMU {
    fn inc_clock(&mut self, cycle_count: u32, cycle_type: Cycle, addr: u32) {
        if cycle_type == Cycle::I { self.clocks_ahead += cycle_count; return }
        self.clocks_ahead += match addr {
            0x00000000 ..= 0x00003FFF => cycle_count,
            _ => unimplemented!("Clock Cycle for {:16X} not implemented!", addr),
        };
    }
}

impl MemoryHandler for MMU {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.read8(addr),
            _ => unimplemented!("Memory Handler for 0x{:16X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.write8(addr, value),
            _ => unimplemented!("Memory Handler for 0x{:16X} not implemented!", addr),
        }
    }
}

pub trait MemoryHandler {
    fn read8(&self, addr: u32) -> u8;
    fn write8(&mut self, addr: u32, value: u8);
    fn read32(&self, addr: u32) -> u32 {
        (self.read8(addr + 0) as u32) << 0 |
        (self.read8(addr + 1) as u32) << 8 |
        (self.read8(addr + 2) as u32) << 16 |
        (self.read8(addr + 3) as u32) << 24
    }
    fn write_u32(&mut self, addr: u32, value: u32) {
        self.write8(addr + 1, (value >> 0) as u8);
        self.write8(addr + 1, (value >> 8) as u8);
        self.write8(addr + 2, (value >> 16) as u8);
        self.write8(addr + 3, (value >> 24) as u8);
    }
}

pub trait IMMU: MemoryHandler {
    fn inc_clock(&mut self, cycle_count: u32, cycle_type: Cycle, addr: u32);
}

#[derive(PartialEq)]
pub enum Cycle {
    N,
    S,
    I,
    // C - No coprocessor in GBA
}
