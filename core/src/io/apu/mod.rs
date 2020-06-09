mod registers;

use super::IORegister;

use registers::*;

pub struct APU {
    soundcnt: SOUNDCNT,
}

impl APU {
    pub fn new() -> APU {
        APU {
            // Registers
            soundcnt: SOUNDCNT::new(),
        }
    }
}

impl APU {
    pub fn read_register(&self, addr: u32) -> u8 {
        match addr {
            0x4000080 => self.soundcnt.read(0),
            0x4000081 => self.soundcnt.read(1),
            0x4000082 => self.soundcnt.read(2),
            0x4000083 => self.soundcnt.read(3),
            _ => { warn!("Ignoring APU Read at 0x{:08X}", addr); 0 },
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        match addr {
            0x4000080 => self.soundcnt.write(0, value),
            0x4000081 => self.soundcnt.write(1, value),
            0x4000082 => self.soundcnt.write(2, value),
            0x4000083 => self.soundcnt.write(3, value),
            _ => warn!("Ignoring APU Write 0x{:08X} = {:02X}", addr, value),
        }
    }
}
