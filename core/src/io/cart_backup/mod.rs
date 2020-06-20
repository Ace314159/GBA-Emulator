use std::fs;
use std::path::PathBuf;

mod eeprom;
mod sram;
mod flash;

use eeprom::EEPROM;
use sram::SRAM;
use flash::Flash;

pub trait CartBackup {
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, value: u8);
    fn read_eeprom(&self, addr: u32) -> u16;
    fn write_eeprom(&mut self, addr: u32, value: u16);
    fn init_eeprom(&mut self, dma_count: u32);

    fn is_dirty(&mut self) -> bool;
    fn get_save_file(&self) -> &PathBuf;
    fn get_mem(&self) -> &Vec<u8>;
    fn is_eeprom(&self) -> bool;
}

impl dyn CartBackup {
    // TODO: Replace with Vec<u8> or array once consts have more features
    const ID_STRINGS: [&'static [u8]; 5] = [
        "EEPROM_V".as_bytes(),
        "SRAM_V".as_bytes(),
        "FLASH_V".as_bytes(),
        "FLASH512_V".as_bytes(),
        "FLASH1M_V".as_bytes(),
    ];

    fn get_type(rom: &Vec<u8>) -> Option<CartBackupType> {
        let mut cart_backup_type = None;
        for rom_start in 0..rom.len() {
            for (id_str_i, id_str) in CartBackup::ID_STRINGS.iter().enumerate() {
                if rom_start + id_str.len() <= rom.len() && rom[rom_start..rom_start + id_str.len()] == **id_str {
                    cart_backup_type = Some(CartBackupType::from(id_str_i));
                    break
                }
            }
        }
        cart_backup_type
    }

    pub fn get(rom: &Vec<u8>, save_file: PathBuf) -> Box<dyn CartBackup> {
        if let Some(cart_backup_type) = CartBackup::get_type(rom) {
            match cart_backup_type {
                CartBackupType::EEPROM => Box::new(EEPROM::new(save_file)),
                CartBackupType::SRAM => Box::new(SRAM::new(save_file)),
                CartBackupType::Flash => Box::new(Flash::new(save_file, 0x10000)),
                CartBackupType::Flash512 => Box::new(Flash::new(save_file, 0x10000)),
                CartBackupType::Flash1M => Box::new(Flash::new(save_file, 0x20000)),
            }
        } else {
            panic!("Unable to Detect Cart Backup Type!");
        }
    }

    pub fn is_eeprom_access(&self, addr: u32, rom_size: usize) -> bool {
        self.is_eeprom() && addr >= 0x0D000000 && addr < 0x0E000000 && (rom_size <= 0x1000000 || addr >= 0x0DFFFF00)
    }

    fn get_initial_mem(save_file: &PathBuf, default_val: u8, size: usize) -> Vec<u8> {
        if let Ok(mem) = fs::read(&save_file) {
            if mem.len() == size { mem } else { vec![default_val; size] }
        } else { vec![default_val; size] }
    }

    pub fn save_to_file(&mut self) {
        if self.is_dirty() { fs::write(self.get_save_file(), self.get_mem()).unwrap() }
    }
}

enum CartBackupType {
    EEPROM = 0,
    SRAM = 1,
    Flash = 2,
    Flash512 = 3,
    Flash1M = 4,
}

impl CartBackupType {
    fn from(value: usize) -> CartBackupType {
        use CartBackupType::*;
        match value {
            0 => EEPROM,
            1 => SRAM,
            2 => Flash,
            3 => Flash512,
            4 => Flash1M,
            _ => panic!("Invalid Cart Backup Type!"),
        }
    }
}
