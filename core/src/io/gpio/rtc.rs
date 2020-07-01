use super::{Event, IORegister, GPIO};
use crate::gba;

pub struct RTC {
    // Pins
    prev_sck: bool,
    sck: bool,
    sio: bool,
    cs: bool,
    // GPIO Registers
    is_used: bool,
    write_only: bool,
    write_mask: u8,
    // RTC Specific
    mode: Mode,
    last_byte: bool,
    counter: usize,
    date_time: DateTime,
}

impl RTC {
    const IDENTIFIER_STRING: &'static [u8] = "SIIRTC_V".as_bytes();
    const COMMAND_CODE: u8 = 0b0110;
    const BIT_REVERSAL: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];

    pub fn new(rom: &Vec<u8>) -> RTC {
        let is_used = (0..(rom.len() - RTC::IDENTIFIER_STRING.len())).any(|i|
            rom[i..(i + RTC::IDENTIFIER_STRING.len())] == *RTC::IDENTIFIER_STRING
        );
        RTC {
            // Pins
            prev_sck: false,
            sck: false,
            sio: false,
            cs: false,
            // GPIO Registers
            is_used,
            write_only: true,
            write_mask: 0b111,
            // RTC Specific
            mode: Mode::StartCommand { done: false },
            counter: gba::CLOCK_FREQ,
            last_byte: false,
            date_time: DateTime::new(),
        }
    }

    fn read_parameter(&mut self, parameter: Parameter) -> (u8, Parameter) {
        let value = match parameter {
            Parameter::Control(byte) => {
                self.last_byte = byte == 0;
                (self.date_time.control.read(), Parameter::Control(byte + 1))
            },
            Parameter::DateTime(byte) => {
                self.last_byte = byte == 6;
                (self.date_time.read(byte), Parameter::DateTime(byte + 1))
            },
            Parameter::Time(byte) => {
                self.last_byte = byte == 2;
                (self.date_time.read(byte + 4), Parameter::Time(byte + 1))
            },
            Parameter::Reset => {
                self.date_time = DateTime::new();
                self.last_byte = true;
                (0, Parameter::Reset)
            }
            Parameter::IRQ => {
                todo!("RTC IRQ");
                // self.last_byte = true;
                // (0, Parameter::IRQ)
            },
        };
        value
    }

    fn write_parameter(&mut self, parameter: Parameter, value: u8) -> Parameter {
        match parameter {
            Parameter::Control(byte) => {
                self.date_time.control.write(value);
                self.last_byte = byte == 0;
                Parameter::Control(byte + 1)
            },
            Parameter::DateTime(byte) => {
                self.date_time.write(byte as u8, value);
                self.last_byte = byte == 6;
                Parameter::DateTime(byte + 1)
            },
            Parameter::Time(byte) => {
                self.date_time.write(byte as u8 + 4, value);
                self.last_byte = byte == 2;
                Parameter::Time(byte + 1)
            },
            Parameter::Reset => {
                self.date_time = DateTime::new();
                self.last_byte = false;
                Parameter::Reset
            },
            Parameter::IRQ => {
                // TODO: RTC Interrupts
                self.last_byte = false;
                Parameter::IRQ
            },
        }
    }
}

impl GPIO for RTC {
    fn clock(&mut self) {
        if !self.is_used { return }
        if self.counter == 0 {
            self.counter = gba::CLOCK_FREQ;
            if self.date_time.second.inc() {
                if self.date_time.minute.inc() {
                    if self.date_time.hour.inc() {
                        self.date_time.day_of_week.inc();
                        // TODO: Use actual number of days in month
                        if self.date_time.day.inc_with_max(30) {
                            if self.date_time.month.inc() {
                                self.date_time.year.inc();
                            }
                        }
                    }
                }
            }
        } else { self.counter -= 1}
    }

    fn process_write(&mut self) {
        self.mode = match self.mode {
            Mode::StartCommand { done: false } => {
                assert!(!self.cs && self.sck);
                Mode::StartCommand { done: true }
            },
            Mode::StartCommand { done: true } if self.cs && self.sck => Mode::SetCommand(0, 0),
            Mode::StartCommand { done: true} => self.mode,

            Mode::SetCommand(command, 7) if self.prev_sck && !self.sck => {
                let command = command | (self.sio as u8) << 7;
                let command = if command & 0xF == RTC::COMMAND_CODE {
                    command >> 4
                } else {
                    debug!("Interpreting MSB RTC Command");
                    assert_eq!(command >> 4, RTC::COMMAND_CODE);
                    RTC::BIT_REVERSAL[((command & 0xF) >> 1) as usize] | (command & 0x1) << 3
                };
                let parameter = Parameter::from(command & 0x7);
                let (parameter, access_type) = if command >> 3 != 0 {
                    let (parameter_byte, next_parameter) = self.read_parameter(parameter);
                    (next_parameter, AccessType::Read(parameter_byte, 0))
                } else { (parameter, AccessType::Write(0, 0)) };
                if parameter == Parameter::Reset || parameter == Parameter::IRQ { Mode::EndCommand }
                else { Mode::ExecCommand(parameter, access_type) }
            },
            Mode::SetCommand(command, bit) if self.prev_sck && !self.sck => {
                assert!(self.cs);
                Mode::SetCommand(command | (self.sio as u8) << bit, bit + 1)
            },
            Mode::SetCommand(_command, _bit) => self.mode,

            Mode::ExecCommand(parameter, AccessType::Read(byte, 7)) if self.prev_sck && !self.sck => {
                let done = self.last_byte;
                self.sio = byte & 0x1 != 0;
                if done { Mode::EndCommand } else {
                    let (parameter_byte, next_parameter) = self.read_parameter(parameter);
                    Mode::ExecCommand(next_parameter, AccessType::Read(parameter_byte, 0))
                }
            }
            Mode::ExecCommand(parameter, AccessType::Read(byte, bit)) if self.prev_sck && !self.sck => {
                self.sio = byte & 0x1 != 0;
                Mode::ExecCommand(parameter, AccessType::Read(byte >> 1, bit + 1))
            },
            Mode::ExecCommand(_parameter, AccessType::Read(_byte, _bit)) => self.mode,

            Mode::ExecCommand(parameter, AccessType::Write(byte, 7)) if self.prev_sck && !self.sck => {
                let done = self.last_byte;
                self.write_parameter(parameter, byte | (self.sio as u8) << 7);
                if done { Mode::EndCommand } else {
                    Mode::ExecCommand(parameter, AccessType::Write(byte + 1, 0))
                }
            },
            Mode::ExecCommand(parameter, AccessType::Write(byte, bit)) if self.prev_sck && !self.sck =>
                Mode::ExecCommand(parameter, AccessType::Write(byte | (self.sio as u8) << bit, bit + 1)),
            Mode::ExecCommand(_parameter, AccessType::Write(_byte, _bit)) => self.mode,

            Mode::EndCommand if !self.cs && self.sck => Mode::StartCommand { done: false },
            Mode::EndCommand => Mode::EndCommand,
        };
    }

    fn set_data0(&mut self, value: bool) {
        assert_eq!(self.write_mask >> 0 & 0x1, 1);
        self.prev_sck = self.sck;
        self.sck = value;
    }
    fn set_data1(&mut self, value: bool) {
        assert_eq!(self.write_mask >> 1 & 0x1, 1);
        self.sio = value;
    }
    fn set_data2(&mut self, value: bool) {
        assert_eq!(self.write_mask >> 2 & 0x1, 1);
        self.cs = value;
    }
    
    fn data0(&self) -> bool { self.sck }
    fn data1(&self) -> bool { self.sio }
    fn data2(&self) -> bool { self.cs }
    
    fn is_used(&self) -> bool { self.is_used }
    fn write_mask(&self) -> u8 { self.write_mask }
    fn can_write(&self, bit: u8) -> bool { self.write_mask >> bit & 0x1 != 0 }
    fn set_write_mask(&mut self, value: u8) { self.write_mask = value }
    fn write_only(&self) -> bool { self.write_only }
    fn set_write_only(&mut self, value: bool) { self.write_only = value }

    fn set_data3(&mut self, _value: bool) { assert_eq!(self.write_mask >> 3 & 0x1, 0) }
    fn data3(&self) -> bool { false }
}

// TODO: Swiwtch implementation to GPIO when adding more GPIO
impl IORegister for RTC {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => {
                let value = 0 |
                if !self.can_write(0) { (self.data0() as u8) << 0 } else { 0 } |
                if !self.can_write(1) { (self.data1() as u8) << 1 } else { 0 } |
                if !self.can_write(2) { (self.data2() as u8) << 2 } else { 0 } |
                if !self.can_write(3) { (self.data3() as u8) << 3 } else { 0 };
                value
            },
            1 => 0,
            2 => self.write_mask(),
            3 => 0,
            4 => self.write_only() as u8,
            5 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => {
                if self.can_write(0) { self.set_data0(value >> 0 & 0x1 != 0) }
                if self.can_write(1) { self.set_data1(value >> 1 & 0x1 != 0) }
                if self.can_write(2) { self.set_data2(value >> 2 & 0x1 != 0) }
                if self.can_write(3) { self.set_data3(value >> 3 & 0x1 != 0) }
                self.process_write();
            },
            1 => (),
            2 => self.set_write_mask(value & 0xF),
            3 => (),
            4 => self.set_write_only(value & 0x1 == 0),
            5 => (),
            _ => unreachable!(),
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    StartCommand { done: bool },
    SetCommand(u8, usize),
    ExecCommand(Parameter, AccessType),
    EndCommand,
}

#[derive(Clone, Copy, Debug)]
enum AccessType {
    Read(u8, usize),
    Write(u8, usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Parameter {
    Control(u8),
    DateTime(u8),
    Time(u8),
    Reset,
    IRQ,
}

impl Parameter {
    pub fn from(value: u8) -> Self {
        match value {
            4 => Parameter::Control(0),
            2 => Parameter::DateTime(0),
            6 => Parameter::Time(0),
            0 => Parameter::Reset,
            3 => Parameter::IRQ,
            _ => panic!("Invalid RTC Command Parameter"),
        }
    }
}

struct Control {
    is_24h: bool,
    per_min_irq: bool,
}

impl Control {
    pub fn new() -> Control {
        Control {
            is_24h: false,
            per_min_irq: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.is_24h as u8) << 6 | (self.per_min_irq as u8) << 3
    }

    pub fn write(&mut self, value: u8) {
        self.is_24h = value >> 6 & 0x1 != 0;
        self.per_min_irq = value >> 3 & 0x1 != 0;
    }
}

struct DateTime {
    control: Control,
    // Date
    year: BCD,
    month: BCD,
    day: BCD,
    day_of_week: BCD,
    // Time
    is_pm: bool,
    hour: BCD,
    minute: BCD,
    second: BCD,
}

impl DateTime {
    pub fn new() -> DateTime {
        DateTime {
            control: Control::new(),
            // Date
            year: BCD::new(0x0, 0x99),
            month: BCD::new(0x1, 0x12),
            day: BCD::new(0x1, 0x30),
            day_of_week: BCD::new(0x1, 0x07),
            // Time
            is_pm: false,
            hour: BCD::new(0x0, 0x23),
            minute: BCD::new(0x0, 0x59),
            second: BCD::new(0x0, 0x59),
        }
    }
}

impl IORegister for DateTime {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.year.value(),
            1 => self.month.value(),
            2 => self.day.value(),
            3 => self.day_of_week.value(),
            4 => {
                let hour = self.hour.value();
                let bit_6 = if self.control.is_24h { hour >= 0x12 } else { self.is_pm };
                (bit_6 as u8) << 6 | hour
            },
            5 => self.minute.value(),
            6 => self.second.value(),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => self.year.set_value(value),
            1 => self.month.set_value(value),
            2 => self.day.set_value(value),
            3 => self.day_of_week.set_value(value),
            4 => {
                self.hour.set_value(value);
                if !self.control.is_24h { self.is_pm = value >> 6 & 0x1 != 0 };
            },
            5 => self.minute.set_value(value),
            6 => self.second.set_value(value),
            _ => unreachable!(),
        }
        None
    }
}

struct BCD {
    initial: u8,
    value: u8,
    max: u8,
}

impl BCD {
    pub fn new(initial: u8, max: u8) -> BCD {
        BCD {
            initial,
            value: initial,
            max,
        }
    }

    pub fn inc(&mut self) -> bool {
        self.inc_with_max(self.max)
    }

    pub fn inc_with_max(&mut self, max: u8) -> bool {
        self.value += 1;
        if self.value > max {
            self.value = self.initial;
            assert!(self.value & 0xF < 0xA && self.value >> 4 < 0xA);
            true
        } else {
            if self.value & 0xF > 0x9 {
                // Shouldn't need to check overflow on upper nibble
                self.value = (self.value & 0xF0) + 0x10;
            }
            assert!(self.value & 0xF < 0xA && self.value >> 4 < 0xA);
            false
        }
    }

    pub fn value(&self) -> u8 { self.value }
    pub fn set_value(&mut self, value: u8) { self.value = value; assert!(self.value & 0xF < 0xA && self.value >> 4 < 0xA) }
}
