use super::super::IORegister;

pub struct Address {
    addr: u32,
    byte3_mask: u32,
}

impl Address {
    pub fn new(any_memory: bool) -> Address {
        Address {
            addr: 0,
            byte3_mask: if any_memory { 0xFF00_0000 } else { 0x7F00_0000 },
        }
    }
}

impl IORegister for Address {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 ..= 3 => (self.addr >> (8 * byte)) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        let mask = 0xFF << (8 * byte);
        match byte {
            0 ..= 2 => self.addr = self.addr & !mask | (value as u32) << (8 * byte) & mask,
            3 => self.addr = self.addr & !mask | (value as u32) << (8 * byte) & self.byte3_mask,
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct WordCount {
    count: u16,
    byte1_mask: u16,
}

impl WordCount {
    pub fn new(is_16bit: bool) -> WordCount {
        WordCount {
            count: 0,
            byte1_mask: if is_16bit { 0xFF00 } else { 0x3F00 },
        }
    }
}

impl IORegister for WordCount {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.count as u8,
            1 => (self.count >> 8) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.count = self.count & !0x00FF | value as u16,
            1 => self.count = self.count & !0xFF00 | (value as u16) << 8 & self.byte1_mask,
            _ => panic!("Invalid Byte!"),
        }
    }
}
