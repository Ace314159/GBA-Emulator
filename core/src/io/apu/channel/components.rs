pub struct Envelope {
    step_time: u8,
    inc: bool,
    initial_volume: u8,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            step_time: 0,
            inc: false,
            initial_volume: 0,
        }
    }

    pub fn read(&self) -> u8 {
        (self.initial_volume << 4) | (self.inc as u8) << 3 | self.step_time
    }

    pub fn write(&mut self, value: u8) {
        self.initial_volume = value >> 4;
        self.inc = value >> 3 & 0x1 != 0;
        self.step_time = value & 0x7;
    }
}
