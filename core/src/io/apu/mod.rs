mod registers;

use super::IORegister;

use registers::*;

pub struct APU {
    cnt: SOUNDCNT,
    bias: SOUNDBIAS,
}

impl APU {
    pub fn new() -> APU {
        APU {
            // Registers
            cnt: SOUNDCNT::new(),
            bias: SOUNDBIAS::new(),
        }
    }
}

impl APU {
    pub fn read_register(&self, addr: u32) -> u8 {
        match addr {
            0x4000080 => self.cnt.read(0),
            0x4000081 => self.cnt.read(1),
            0x4000082 => self.cnt.read(2),
            0x4000083 => self.cnt.read(3),
            0x4000088 => self.bias.read(0),
            0x4000089 => self.bias.read(1),
            _ => { warn!("Ignoring APU Read at 0x{:08X}", addr); 0 },
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        match addr {
            0x4000080 => self.cnt.write(0, value),
            0x4000081 => self.cnt.write(1, value),
            0x4000082 => self.cnt.write(2, value),
            0x4000083 => self.cnt.write(3, value),
            0x4000088 => self.bias.write(0, value),
            0x4000089 => self.bias.write(1, value),
            _ => warn!("Ignoring APU Write 0x{:08X} = {:02X}", addr, value),
        }
    }
}
