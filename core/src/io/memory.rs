use super::MemoryHandler;

pub struct ROM {
    mem: Vec<u8>,
    offset: usize,
}

impl ROM {
    pub fn new(start_addr: u32, mem: Vec<u8>) -> ROM {
        ROM {
            mem,
            offset: start_addr as usize,
        }
    }
}

impl MemoryHandler for ROM {
    fn read8(&self, addr: u32) -> u8 {
        let addr = addr as usize - self.offset;
        if addr < self.mem.len() { self.mem[addr] }
        else { warn!("Ignoring ROM Read at 0x{:08X}", addr); 0 }
    }

    fn write8(&mut self, _addr: u32, _value: u8) {}
}

pub struct RAM {
    mem: Vec<u8>,
    offset: usize,
}

impl RAM {
    pub fn new(start_addr: u32, size: usize, initial_value: u8) -> RAM {
        RAM {
            mem: vec![initial_value; size],
            offset: start_addr as usize,
        }
    }
}

impl MemoryHandler for RAM {
    fn read8(&self, addr: u32) -> u8 {
        self.mem[addr as usize - self.offset]
    }

    fn write8(&mut self, addr: u32, value: u8) {
        self.mem[addr as usize - self.offset] = value;
    }
}
