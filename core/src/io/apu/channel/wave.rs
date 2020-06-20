use super::components::*;

use super::{Channel, IORegister};

pub struct Wave {
    // Registers
    use_two_banks: bool,
    wave_ram_bank: u8,
    enabled: bool,
    length_reload: u8,
    volume: u8,
    force_volume: bool,
    sample_rate: u16,
    use_length: bool,
    // Sound Generation
    pub length_counter: LengthCounter,
    pub wave_ram: [[u8; 16]; 2],
    wave_ram_i: usize,
    timer: Timer<u16>,
}

impl Wave {
    const VOLUME_FACTORS: [i16; 4] = [0, 8, 4, 2];

    pub fn new() -> Wave {
        Wave {
            // Registers
            use_two_banks: false,
            wave_ram_bank: 0,
            enabled: false,
            length_reload: 0,
            volume: 0,
            force_volume: false,
            sample_rate: 0,
            use_length: false,
            // Sound Generation
            length_counter: LengthCounter::new(),
            wave_ram: [[0; 16]; 2],
            wave_ram_i: 0,
            timer: Timer::new(8 * 2048),
        }
    }

    fn calc_reload(&self) -> u16 {
        8 * (2048 - self.sample_rate)
    }

    pub fn clock(&mut self) {
        if self.timer.clock_with_reload(self.calc_reload()) {
            self.wave_ram_i += 1;
            if self.wave_ram_i == 32 {
                self.wave_ram_i = 0;
                if self.use_two_banks { self.wave_ram_bank ^= 1 }
            }
        }
    }

    pub fn read_wave_ram(&self, offset: u32) -> u8 {
        self.wave_ram[(self.wave_ram_bank as usize) ^ 1][offset as usize]
    }

    pub fn write_wave_ram(&mut self, offset: u32, value: u8) {
        self.wave_ram[(self.wave_ram_bank as usize) ^ 1][offset as usize] = value;
    }
}

impl Channel for Wave {
    fn generate_sample(&self) -> i16 {
        let byte = self.wave_ram[self.wave_ram_bank as usize][self.wave_ram_i / 2];
        let sample = if self.wave_ram_i % 2 == 0 { byte >> 4 } else { byte & 0xF };
        let volume_factor = if self.force_volume { 6 } else { Wave::VOLUME_FACTORS[self.volume as usize] };
        volume_factor * (sample as i16)
    }

    fn is_on(&self) -> bool {
        self.enabled && (!self.use_length || self.length_counter.should_play())
    }
}

impl IORegister for Wave {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.enabled as u8) << 7 | self.wave_ram_bank << 6 | (self.use_two_banks as u8) << 5,
            1 => 0,
            2 => 0,
            3 => self.volume << 5,
            4 => 0,
            5 => (self.use_length as u8) << 6,
            6 | 7 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.enabled = value >> 7 & 0x1 != 0;
                self.wave_ram_bank = value >> 6 & 0x1;
                self.use_two_banks = value >> 5 & 0x1 != 0;
            },
            1 => (),
            2 => self.length_reload = value,
            3 => {
                self.volume = value >> 5 & 0x3;
                self.force_volume = value >> 7 & 0x1 != 0;
            },
            4 => self.sample_rate = self.sample_rate & !0xFF | value as u16,
            5 => {
                self.sample_rate = self.sample_rate & !0x700 | ((value & 0x7) as u16) << 8;
                self.use_length = value >> 6 & 0x1 != 0;
                if value & 0x80 != 0 {
                    self.wave_ram_i = 0;
                    self.timer.reload(self.calc_reload());
                    self.length_counter.reload_length(256 - self.length_reload as u16);
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
    }
}
