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
    clock: u8,
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
            clock: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.step_period == 0 { return }
        if self.clock == 0 {
            if self.inc {
                assert!(self.cur_volume <= 15);
                if self.cur_volume != 15 { self.cur_volume += 1; self.clock = self.step_period }
            } else {
                if self.cur_volume != 0 { self.cur_volume -= 1; self.clock = self.step_period }
            }
        } else { self.clock -= 1 }
    }

    pub fn get_volume(&self) -> f32 {
        self.cur_volume as f32
    }

    pub fn reset(&mut self) {
        self.cur_volume = self.initial_volume;
        self.clock = self.step_period;
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
