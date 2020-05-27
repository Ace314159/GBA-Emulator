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
        if addr > self.mem.len() { 0 }
        else { self.mem[addr] }
    }

    fn write8(&mut self, _addr: u32, _value: u8) {}
}

pub struct RAM {
    mem: Vec<u8>,
    offset: usize,
}

impl RAM {
    pub fn new(start_addr: u32, size: usize) -> RAM {
        RAM {
            mem: vec![0; size],
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
