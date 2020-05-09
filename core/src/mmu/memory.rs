use super::MemoryHandler;

pub struct ROM {
    mem: Vec<u8>,
}

impl ROM {
    pub fn new(mem: Vec<u8>) -> ROM {
        ROM {
            mem,
        }
    }
}

impl MemoryHandler for ROM {
    fn read8(&self, addr: u32) -> u8 {
        self.mem[addr as usize]
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
