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
        else { warn!("Returning Invalid ROM Read at 0x{:08X}", addr); 0 }
    }

    fn write8(&mut self, _addr: u32, _value: u8) {}
}
