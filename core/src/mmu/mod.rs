pub struct MMU {
    bios: Vec<u8>,
    rom: Vec<u8>,
    clocks_ahead: u32,
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            bios,
            rom,
            clocks_ahead: 0,
        }
    }
    
    fn read_u32(buffer: &Vec<u8>, addr: u32) -> u32 {
        let addr = addr as usize;
        (buffer[addr + 0] as u32) |
        (buffer[addr + 1] as u32) << 8 |
        (buffer[addr + 2] as u32) << 16 |
        (buffer[addr + 3] as u32) << 24
    }
}

impl IMMU for MMU {
    fn read32(&self, addr: u32) -> u32 {
        match addr {
            0x00000000 ..= 0x00003FFF => MMU::read_u32(&self.bios, addr),
            _ => unimplemented!("Read from {:16X} not implemented!", addr),
        }
    }

    fn write32(&mut self, addr: u32, value: u32) {
        unimplemented!("Write to {:16X} not implemented!", addr)
    }

    fn inc_clock(&mut self, cycle_count: u32, cycle_type: Cycle, addr: u32) {
        self.clocks_ahead += match addr {
            0x00000000 ..= 0x00003FFF => cycle_count,
            _ => unimplemented!("Clock Cycle for {:16X} not implemented!", addr),
        };
    }
}

pub trait IMMU {
    fn read32(&self, addr: u32) -> u32;
    fn write32(&mut self, addr: u32, value: u32);
    fn inc_clock(&mut self, cycle_count: u32, cycle_type: Cycle, addr: u32);
}

pub enum Cycle {
    N,
    S,
    I,
    // C,
}
