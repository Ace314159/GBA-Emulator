use super::components::*;

use super::{Channel, Scheduler, IORegister};

pub struct Noise {
    // Registers
    length_reload: u8,
    pub envelope: Envelope,
    ratio: u8,
    counter_width: bool,
    shift: u8,
    use_length: bool,
    // Sound Generation
    pub length_counter: LengthCounter,
    timer: Timer<u16>,
    lfsr: u16,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            // Registers
            length_reload: 0,
            envelope: Envelope::new(),
            ratio: 0,
            counter_width: false,
            shift: 0,
            use_length: false,
            // Sound Generation
            length_counter: LengthCounter::new(),
            timer: Timer::new(8),
            lfsr: 0x7FFF,
        }
    }

    fn calc_reload(&self) -> u16 {
        let interval = 64 << self.shift;
        if self.ratio == 0 { interval / 2 } else { interval * self.ratio as u16 }
    }

    pub fn clock(&mut self) {
        if !self.is_on() { return }
        let reload = self.calc_reload();
        if reload != 0 && self.timer.clock_with_reload(reload) {
            let carry = self.lfsr & 0x1 != 0;
            self.lfsr >>= 1;
            if carry { self.lfsr ^= [0x6000, 0x60][self.counter_width as usize] }
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
            4 => self.shift << 4 | (self.counter_width as u8) << 3 | self.ratio,
            5 => (self.use_length as u8) << 6,
            6 | 7 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, _scheduler: &mut Scheduler, byte: u8, value: u8) {
        match byte {
            0 => self.length_reload = value & 0x3F,
            1 => self.envelope.write(value),
            2 | 3 => (),
            4 => {
                self.ratio = value & 0x7;
                self.counter_width = value >> 3 & 0x1 != 0;
                self.shift = value >> 4 & 0xF;
            },
            5 => {
                self.use_length = value >> 6 & 0x1 != 0;
                if value & 0x80 != 0 {
                    self.lfsr = 0x7FFF;
                    let reload = self.calc_reload();
                    if reload != 0 {
                        self.timer.reload(reload);
                    }
                    self.envelope.reset();
                    self.length_counter.reload_length(64 - self.length_reload as u16);
                }
            },
            6 | 7 => (),
            _ => unreachable!(),
        }
    }
}
