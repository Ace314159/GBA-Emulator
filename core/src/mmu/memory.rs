use super::MemoryHandler;

pub struct ROM {
    mem: Vec<u8>
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
