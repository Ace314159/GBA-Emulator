use num_traits as num;
use num::{NumAssign, Unsigned};

pub struct Sweep {
    // Registers
    shift: u8,
    negate: bool,
    period: u8,
    // Sound Generation
    enabled: bool,
    clock: u8,
    pub freq: u16,
    freq_shadow: u16,
}

impl Sweep {
    pub fn new() -> Sweep {
        Sweep {
            // Registers
            shift: 0,
            negate: false,
            period: 0,
            // Sound Generation
            enabled: false,
            clock: 0,
            freq: 0,
            freq_shadow: 0,
        }
    }

    pub fn clock(&mut self) {
        if !self.enabled { return }
        if self.clock == 0 {
            if self.period != 0 {
                let new_freq = self.calc_new_freq();
                if !self.overflow_check(new_freq) {
                    self.freq = new_freq;
                    self.freq_shadow = new_freq;
                    self.overflow_check(self.calc_new_freq());
                }
            }
            self.clock = self.period;
        } else { self.clock -= 1 }
    }

    pub fn reload(&mut self) {
        self.freq_shadow = self.freq;
        self.clock = self.period;
        self.enabled = self.period != 0 || self.shift != 0;
        if self.shift != 0 {
            self.overflow_check(self.calc_new_freq());
        }
    }

    pub fn read(&self) -> u8 {
        self.period << 4 | (self.negate as u8) << 3 | self.shift
    }
    
    pub fn write(&mut self, value: u8) {
        self.shift = value & 0x7;
        self.negate = value >> 3 & 0x1 != 0;
        self.period = value >> 4 & 0x7;
    }

    fn calc_new_freq(&self) -> u16 {
        let operand = self.freq_shadow >> self.shift;
        if self.negate {
            self.freq_shadow.wrapping_add(!operand).wrapping_add(1)
        } else {
            self.freq_shadow.wrapping_add(operand)
        }
    }

    fn overflow_check(&mut self, new_freq: u16) -> bool {
        if new_freq >= 0x800 {
            self.enabled = false;
            true
        } else { false }
    }
}

pub struct LengthCounter {
    // Registers
    length: u16,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            length: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.length != 0 { self.length -= 1 }
    }

    pub fn reload_length(&mut self, length: u16) {
        self.length = length;
    }

    pub fn should_play(&self) -> bool {
        self.length != 0
    }
}

pub struct Envelope {
    // Registers
    step_period: u8,
    inc: bool,
    initial_volume: u8,
    // Sound Generation
    cur_volume: u8,
    timer: Timer<u8>,
    active: bool,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            // Registers
            step_period: 0,
            inc: false,
            initial_volume: 0,
            // Sound Generation
            cur_volume: 0,
            timer: Timer::new(1),
            active: false,
        }
    }

    pub fn clock(&mut self) {
        if self.step_period == 0 || !self.active { return }
        if self.timer.clock_with_reload(self.step_period) {
            if self.inc {
                assert!(self.cur_volume <= 15);
                if self.cur_volume == 15 { self.active = false }
                else { self.cur_volume += 1 }
            } else {
                if self.cur_volume == 0 { self.active = false }
                else { self.cur_volume -= 1 }
            }
        }
    }

    pub fn get_volume(&self) -> f32 {
        self.cur_volume as f32
    }

    pub fn reset(&mut self) {
        self.cur_volume = self.initial_volume;
        if self.step_period != 0 { self.timer.reload(self.step_period) }
        self.active = true;
    }

    pub fn read(&self) -> u8 {
        (self.initial_volume << 4) | (self.inc as u8) << 3 | self.step_period
    }

    pub fn write(&mut self, value: u8) {
        self.initial_volume = value >> 4;
        self.inc = value >> 3 & 0x1 != 0;
        self.step_period = value & 0x7;
    }
}

pub struct Timer<T: NumAssign + Unsigned + Copy> {
    counter: T,
    reload: T,
}

impl<T: NumAssign + Unsigned + Copy> Timer<T> {
    pub fn new(reload: T) -> Timer<T> {
        assert!(reload != num::zero());
        Timer {
            counter: reload,
            reload,
        }
    }

    pub fn clock(&mut self) -> bool {
        self.counter -= num::one();
        if self.counter == num::one() {
            self.counter = self.reload;
            true
        } else { false }
    }

    pub fn clock_with_reload(&mut self, reload: T) -> bool {
        self.reload(reload);
        self.clock()
    }

    pub fn reload(&mut self, reload: T) {
        assert!(reload != num::zero());
        self.reload = reload
    }
}
