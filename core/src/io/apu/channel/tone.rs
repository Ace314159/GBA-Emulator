use super::components::*;

use super::IORegister;
use super::Channel;

pub struct Tone {
    // Registers
    pub sweep: Sweep,
    length_data: u8,
    duty: u8,
    pub envelope: Envelope,
    use_length: bool,

    // Sound Generation
    pub length_counter: LengthCounter,
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
            sweep: Sweep::new(),
            length_data: 0,
            duty: 0,
            envelope: Envelope::new(),
            use_length: false,

            // Sound Generation
            length_counter: LengthCounter::new(),
            duty_clock: 0,
            period: 0,
            cur_duty: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.duty_clock == 0 {
            self.duty_clock = 16;
            if self.period == 0 {
                self.period = 2048 - self.sweep.freq;
                self.cur_duty = (self.cur_duty + 1) % 8;
            } else { self.period -= 1 }
        } else { self.duty_clock -= 1}
    }
}

impl Channel for Tone {
    fn generate_sample(&self) -> f32 {
        if !self.use_length || self.length_counter.should_play() {
            self.envelope.get_volume() * Tone::DUTY[self.duty as usize][self.cur_duty]
        } else { 0.0 }
    }

    fn is_on(&self) -> bool {
        !self.use_length || self.length_counter.should_play()
    }
}

impl IORegister for Tone {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.sweep.read(),
            1 => 0,
            2 => self.duty << 6,
            3 => self.envelope.read(),
            4 => 0,
            5 => (self.use_length as u8) << 6,
            6 | 7 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.sweep.write(value),
            1 => (),
            2 => {
                self.duty = value >> 6 & 0x3;
                self.length_data = value & 0x3F;
            },
            3 => self.envelope.write(value),
            4 => self.sweep.freq = self.sweep.freq & !0xFF | value as u16,
            5 => {
                self.sweep.freq = self.sweep.freq & !0x700 | ((value & 0x7) as u16) << 8;
                self.use_length = value >> 6 & 0x1 != 0;
                if value & 0x80 != 0 {
                    self.sweep.reload();
                    self.cur_duty = 0;
                    self.period = 2048 - self.sweep.freq;
                    self.envelope.reset();
                    self.length_counter.reload_length(64 - self.length_data as u16);
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
    }
}
