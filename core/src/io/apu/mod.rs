mod registers;

use registers::*;

pub struct APU {

}

impl APU {
    pub fn new() -> APU {
        APU {

        }
    }
}

impl APU {
    pub fn read_register(&self, addr: u32) -> u8 {
        match addr {
            _ => { warn!("Ignoring APU Read at 0x{:08X}", addr); 0 },
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        match addr {
            _ => warn!("Ignoring APU Write 0x{:08X} = {:02X}", addr, value),
        }
    }
}
