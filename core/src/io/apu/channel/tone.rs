use super::components::*;

use super::IORegister;
use super::Channel;

pub struct Tone {
    // Registers
    length: u8,
    duty: u8,
    pub envelope: Envelope,
    freq_raw: u16,
    use_length: bool,

    // Sound Generation
    duty_clock: u8,
    period: u16,
    cur_duty: usize,
}

impl Tone {
    const DUTY: [[f32; 8]; 4] = [
        [1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0],
        [1.0,  1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0],
        [1.0,  1.0,  1.0,  1.0, -1.0, -1.0, -1.0, -1.0],
        [1.0,  1.0,  1.0,  1.0,  1.0,  1.0, -1.0, -1.0],
    ];

    pub fn new() -> Tone {
        Tone {
            // Registers
            length: 0,
            duty: 0,
            envelope: Envelope::new(),
            freq_raw: 0,
            use_length: false,

            // Sound Generation
            duty_clock: 0,
            period: 0,
            cur_duty: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.duty_clock == 0 {
            self.duty_clock = 16;
            if self.period == 0 {
                self.period = 2048 - self.freq_raw;
                self.cur_duty = (self.cur_duty + 1) % 8;
            } else { self.period -= 1 }
        } else { self.duty_clock -= 1}
    }
}

impl Channel for Tone {
    fn generate_sample(&self) -> f32 {
        self.envelope.get_volume() * Tone::DUTY[self.duty as usize][self.cur_duty]
    }
}

impl IORegister for Tone {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.duty << 6 | self.length,
            1 => self.envelope.read(),
            2 | 3 => 0,
            4 => 0,
            5 => (self.use_length as u8) << 6,
            6 | 7 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.duty = value >> 6 & 0x3;
                self.length = value & 0x3F;
            },
            1 => self.envelope.write(value),
            2 | 3 => (),
            4 => self.freq_raw = self.freq_raw & !0xFF | value as u16,
            5 => {
                self.freq_raw = self.freq_raw & !0x700 | ((value & 0x7) as u16) << 8;
                self.use_length = value >> 6 & 0x1 != 0;
                if value & 0x80 != 0 {
                    self.cur_duty = 0;
                    self.period = 2048 - self.freq_raw;
                    self.envelope.reset();
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
    }
}
