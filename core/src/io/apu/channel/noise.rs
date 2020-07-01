use super::components::*;

use super::{Channel, Event, IORegister};

pub struct Noise {
    // Registers
    length_reload: u8,
    pub envelope: Envelope,
    divisor_code: u8,
    counter_7bit: bool,
    clock_shift: u8,
    use_length: bool,
    // Sound Generation
    pub length_counter: LengthCounter,
    timer: Timer<u16>,
    lfsr: u16,
}

impl Noise {
    const DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

    pub fn new() -> Noise {
        Noise {
            // Registers
            length_reload: 0,
            envelope: Envelope::new(),
            divisor_code: 0,
            counter_7bit: false,
            clock_shift: 0,
            use_length: false,
            // Sound Generation
            length_counter: LengthCounter::new(),
            timer: Timer::new(8),
            lfsr: 0x7FFF,
        }
    }

    fn calc_reload(&self) -> u16 {
        std::cmp::max(1, 4 * (Noise::DIVISORS[self.divisor_code as usize] << self.clock_shift))
    }

    pub fn clock(&mut self) {
        if !self.is_on() { return }
        if self.timer.clock_with_reload(self.calc_reload()) {
            let new_high = (self.lfsr & 0x1) ^ (self.lfsr >> 1 & 0x1);
            self.lfsr = new_high << 14 | self.lfsr >> 1;
            if self.counter_7bit { self.lfsr = self.lfsr & !0x40 | new_high << 6 }
        }
    }
}

impl Channel for Noise {
    fn generate_sample(&self) -> i16 {
        if self.is_on() {
            // Multiply by 8, so that wave ram volume can be more easily changed
            self.envelope.get_volume() * [-8, 8][(!self.lfsr & 0x1) as usize] as i16
        } else { 0 }
    }

    fn is_on(&self) -> bool {
        !self.use_length || self.length_counter.should_play()
    }
}

impl IORegister for Noise {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => 0,
            1 => self.envelope.read(),
            2 | 3 => 0,
            4 => self.clock_shift << 4 | (self.counter_7bit as u8) << 3 | self.divisor_code,
            5 => (self.use_length as u8) << 6,
            6 | 7 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => self.length_reload = value & 0x3F,
            1 => self.envelope.write(value),
            2 | 3 => (),
            4 => {
                self.divisor_code = value & 0x7;
                self.counter_7bit = value >> 3 & 0x1 != 0;
                self.clock_shift = value >> 4 & 0xF;
            },
            5 => {
                self.use_length = value >> 6 & 0x1 != 0;
                if value & 0x80 != 0 {
                    self.lfsr = 0x7FFF;
                    self.timer.reload(self.calc_reload());
                    self.envelope.reset();
                    self.length_counter.reload_length(64 - self.length_reload as u16);
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
        None
    }
}
