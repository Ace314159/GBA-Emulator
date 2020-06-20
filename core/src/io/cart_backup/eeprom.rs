use std::path::PathBuf;
use std::cell::Cell;

use super::CartBackup;

pub struct EEPROM {
    mem: Vec<u8>,
    mem_size: usize,
    save_file: PathBuf,
    is_dirty: bool,
    
    
    addr_size: usize,
    mode: Cell<Mode>,
}

impl EEPROM {
    pub fn new(save_file: PathBuf) -> EEPROM {
        EEPROM {
            mem: Vec::new(),
            mem_size: 0,
            save_file,
            is_dirty: false,
            
            addr_size: 0,
            mode: Cell::new(Mode::Request { done: false }),
        }
    }
}

impl CartBackup for EEPROM {
    fn init_eeprom(&mut self, dma_count: u32) {
        if self.mem_size == 0 {
            assert!(dma_count == 9 || dma_count == 17);
            self.addr_size = dma_count as usize - 3; // Number of bits
            self.mem_size = 1 << (self.addr_size + 3); // Number of bytes
            self.mem = CartBackup::get_initial_mem(&self.save_file, 0, self.mem_size);
        }
    }

    fn read_eeprom(&self, _addr: u32) -> u16 {
        let (new_mode, bit) = match self.mode.get() {
            Mode::Read(IOMode::Data(1, addr)) => {
                let byte = (64 - 1) / 8;
                let bit_num = 7 - ((64 - 1) % 8); // MSB first
                let val = self.mem[addr + byte] >> bit_num & 0x1;
                (Mode::Request { done: false }, val)
            },
            Mode::Read(IOMode::Data(counter, addr)) if counter > 64 => 
                (Mode::Read(IOMode::Data(counter - 1, addr)), 0),
            Mode::Read(IOMode::Data(counter, addr)) => {
                let byte = (64 - counter) / 8;
                let bit_num = 7 - ((64 - counter) % 8); // MSB first
                let val = self.mem[addr + byte] >> bit_num & 0x1;
                (Mode::Read(IOMode::Data(counter - 1, addr)), val)
            },
            _ => unreachable!(),
        };
        assert!(bit == 0 || bit == 1);
        self.mode.set(new_mode);
        bit as u16
    }

    fn write_eeprom(&mut self, _addr: u32, value: u16) {
        let bit = (value & 0x1) as usize;
        let mode = self.mode.get();
        let new_mode = match mode {
            Mode::Request { done: false } => {
                assert_eq!(bit, 1);
                Mode::Request { done: true }
            },
            Mode::Request { done: true } => {
                if bit == 1 { Mode::Read(IOMode::Address(self.addr_size, 0)) }
                else { Mode::Write(IOMode::Address(self.addr_size - 1, 0),) }
            },

            Mode::Read(IOMode::Address(0, addr)) => Mode::Read(IOMode::Data(64 + 4, addr * 8)),
            Mode::Read(IOMode::Data(_counter, _addr)) => unreachable!(),

            Mode::Write(IOMode::Address(0, addr)) => Mode::Write(IOMode::Data(64, (addr << 1 | bit) * 8)),
            Mode::Write(IOMode::Data(0, _addr)) => {
                assert_eq!(bit, 0);
                Mode::Request { done: false }
            },
            Mode::Write(IOMode::Data(counter, addr)) => {
                self.is_dirty = true;
                let byte = (64 - counter) / 8;
                let bit_num = 7 - ((64 - counter) % 8); // MSB first
                self.mem[addr + byte] = self.mem[addr + byte] & !(1 << bit_num) | (bit as u8) << bit_num;
                Mode::Write(IOMode::Data(counter - 1, addr))
            },

            Mode::Read(IOMode::Address(counter, addr)) |
            Mode::Write(IOMode::Address(counter, addr)) => {
                match mode {
                    Mode::Read {..} => Mode::Read(IOMode::Address(counter - 1, addr << 1 | bit)),
                    Mode::Write {..} => Mode::Write(IOMode::Address(counter - 1, addr << 1 | bit)),
                    _ => unreachable!()
                }
            },
        };
        self.mode.set(new_mode);
    }

    fn read(&self, _addr: u32) -> u8 { unreachable!() }
    fn write(&mut self, _addr: u32, _value: u8) { unreachable!() }
    fn is_dirty(&mut self) -> bool { let is_dirty = self.is_dirty; self.is_dirty = false; is_dirty }
    fn get_save_file(&self) -> &PathBuf { &self.save_file }
    fn get_mem(&self) -> &Vec<u8> { &self.mem }
    fn is_eeprom(&self) -> bool { true }
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    Request { done: bool },
    Read(IOMode),
    Write(IOMode),
}

#[derive(Clone, Copy, Debug)]
enum IOMode {
    Address(usize, usize),
    Data(usize, usize),
}
