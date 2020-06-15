use super::components::*;

use super::{Channel, IORegister};

pub struct Tone {
    // Registers
    pub sweep: Sweep,
    length_data: u8,
    duty: u8,
    pub envelope: Envelope,
    use_length: bool,

    // Sound Generation
    pub length_counter: LengthCounter,
    timer: Timer<u16>,
    duty_pos: usize,
}

impl Tone {
    // Filled with 8s so that wave ram volume can be more easily changed
    const DUTY: [[i16; 8]; 4] = [
        [8, -8, -8, -8, -8, -8, -8, -8],
        [8,  8, -8, -8, -8, -8, -8, -8],
        [8,  8,  8,  8, -8, -8, -8, -8],
        [8,  8,  8,  8,  8,  8, -8, -8],
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
            timer: Timer::new(16 * 2048),
            duty_pos: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer.clock_with_reload(16 * (2048 - self.sweep.freq)) {
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }
}

impl Channel for Tone {
    fn generate_sample(&self) -> i16 {
        if !self.use_length || self.length_counter.should_play() {
            self.envelope.get_volume() * Tone::DUTY[self.duty as usize][self.duty_pos]
        } else { 0 }
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
                    self.duty_pos = 0;
                    self.timer.reload(16 * (2048 - self.sweep.freq));
                    self.envelope.reset();
                    self.length_counter.reload_length(64 - self.length_data as u16);
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
    }
}
