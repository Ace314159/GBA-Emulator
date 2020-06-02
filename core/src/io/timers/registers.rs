use super::IORegister;

pub struct TMCNT {
    period: u16, // Parsed value of prescaler_selection
    prescaler_selection: u8,
    count_up: bool,
    irq: bool,
    start: bool,
}

impl IORegister for TMCNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.start as u8) << 7 | (self.irq as u8) << 6 | (self.count_up as u8) << 2 | self.prescaler_selection,
            1 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.start = value >> 7 & 0x1 != 0;
                self.irq = value >> 6 & 0x1 != 0;
                self.count_up = value >> 2 & 0x1 != 0;
                self.prescaler_selection = value & 0x3;
                self.period = match self.prescaler_selection {
                    0 => 1,
                    1 => 64,
                    2 => 256,
                    3 => 1024,
                    _ => unreachable!(),
                }
            },
            1 => (),
            _ => unreachable!(),
        }
    }
}

impl TMCNT {
    pub fn new() -> TMCNT {
        TMCNT {
            period: 1, // Parsed value of prescaler_selection
            prescaler_selection: 0,
            count_up: false,
            irq: false,
            start: false,
        }
    }
}
