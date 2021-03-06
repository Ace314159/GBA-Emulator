use std::path::PathBuf;

use super::CartBackup;

pub struct Flash {
    mem: Vec<u8>,
    mem_size: usize,
    save_file: PathBuf,
    is_dirty: bool,

    command: Command,
    mode: Mode,
    bank: usize,
    in_chip_ident: bool,
}

impl Flash {
    // Sanyo Manufacturer and Device IDs
    const MANUFACTURER_ID: u8 = 0x62;
    const DEVICE_ID: u8 = 0x13;

    const COMMAND_ADDR: u32 = 0x5555;
    const COMMAND1_ADDR: u32 = 0x2AAA;

    pub fn new(save_file: PathBuf, size: usize) -> Flash {
        Flash {
            mem: CartBackup::get_initial_mem(&save_file, 0xFF, size),
            mem_size: size,
            save_file,
            is_dirty: false,

            command: Command::Command0,
            mode: Mode::Ready,
            bank: 0,
            in_chip_ident: false,
        }
    }
}

impl CartBackup for Flash {
    fn read(&self, addr: u32) -> u8 {
        if self.in_chip_ident {
            if addr == 0 { Flash::MANUFACTURER_ID }
            else if addr == 1 { Flash::DEVICE_ID }
            else { self.mem[self.bank * 0x10000 + addr as usize] }
        } else { self.mem[self.bank * 0x10000 + addr as usize] }
    }

    fn write(&mut self, addr: u32, value: u8) {
        if self.mode == Mode::Write {
            self.is_dirty = true;
            self.mem[self.bank * 0x10000 + addr as usize] = value;
            self.mode = Mode::Ready;
            return
        } else if self.mode == Mode::SetBank {
            assert_eq!(addr, 0);
            assert!(value == 0 || value == 1);
            self.bank = value as usize;
            self.mode = Mode::Ready;
            return
        }

        match self.command {
            Command::Command0 => {
                assert_eq!(addr, Flash::COMMAND_ADDR);
                if value != 0xAA { return }
                self.command = Command::Command1;
                return
            },
            Command::Command1 => {
                assert_eq!(addr, Flash::COMMAND1_ADDR);
                assert_eq!(value, 0x55);
                self.command = Command::Command2;
                return
            },
            Command::Command2 => {
                self.command = Command::Command0;
            },
        };
        match self.mode {
            Mode::Ready => {
                assert_eq!(addr, Flash::COMMAND_ADDR);
                self.mode = match value {
                    0x90 => { self.in_chip_ident = true; Mode::Ready },
                    0xF0 => { self.in_chip_ident = false; Mode::Ready },
                    0x80 => Mode::Erase,
                    0xA0 => Mode::Write,
                    0xB0 => Mode::SetBank,
                    _ => panic!("Invalid Command: {:X}", value),
                };
            },
            Mode::Erase => {
                match value {
                    0x10 => { assert_eq!(addr, Flash::COMMAND_ADDR); self.is_dirty = true; self.mem = vec![0xFF; self.mem_size] },
                    0x30 => {
                        assert_eq!(addr & !0xF000, 0);
                        let sector = addr as usize;
                        self.is_dirty = true;
                        for i in self.mem[sector..sector + 0x1000].iter_mut() { *i = 0xFF }
                    }
                    _ => panic!("Invalid Erase Command: {:X}", value),
                };
                self.mode = Mode::Ready;
            },
            _ => unreachable!(),
        };
    }

    fn init_eeprom(&mut self, _dma_count: u32) {}
    fn read_eeprom(&self, _addr: u32) -> u16 { unreachable!() }
    fn write_eeprom(&mut self, _addr: u32, _value: u16) { unreachable!() }
    fn is_dirty(&mut self) -> bool { let is_dirty = self.is_dirty; self.is_dirty = false; is_dirty }
    fn get_save_file(&self) -> &PathBuf { &self.save_file }
    fn get_mem(&self) -> &Vec<u8> { &self.mem }
    fn is_eeprom(&self) -> bool { false }
}

#[derive(Debug, PartialEq)]
enum Command {
    Command0,
    Command1,
    Command2,
}

#[derive(Debug, PartialEq)]
enum Mode {
    Ready,
    Erase,
    Write,
    SetBank,
}
