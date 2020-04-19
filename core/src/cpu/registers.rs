#[derive(Clone, Copy)]
pub enum Reg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13, // SP
    R14 = 14, // LR
    R15 = 15, // PC
    CPSR,
    SPSR,
}

pub enum Mode {
    USR = 0b10000,
    FIQ = 0b10001,
    IRQ = 0b10010,
    SVC = 0b10011,
    ABT = 0b10111,
    SYS = 0b11111,
    UND = 0b11011,
}

bitflags! {
    struct StatusReg: u32 {
        const N = 0x8000;
        const Z = 0x4000;
        const C = 0x2000;
        const V = 0x1000;
        const I = 0x0080;
        const F = 0x0040;
        const T = 0x0020;
        const M4 = 0x0010;
        const M3 = 0x0008;
        const M2 = 0x0004;
        const M1 = 0x0002;
        const M0 = 0x0001;
    }
}

impl StatusReg {
    pub fn reset() -> StatusReg {
        StatusReg::I | StatusReg::F | StatusReg::from_bits(Mode::SVC as u32).unwrap()
    }

    pub fn get_mode(&self) -> Mode {
        match Some(self.bits() & 0x1F) {
            Some(m) if m == Mode::USR as u32 => Mode::USR,
            Some(m) if m == Mode::FIQ as u32 => Mode::FIQ,
            Some(m) if m == Mode::IRQ as u32 => Mode::IRQ,
            Some(m) if m == Mode::SVC as u32 => Mode::SVC,
            Some(m) if m == Mode::ABT as u32 => Mode::ABT,
            Some(m) if m == Mode::SYS as u32 => Mode::SYS,
            Some(m) if m == Mode::UND as u32 => Mode::UND,
            _ => panic!("Invalid Mode"),
        }
    }
}

pub struct RegValues {
    usr: [u32; 16],
    fiq: [u32; 7],
    svc: [u32; 2],
    abt: [u32; 2],
    irq: [u32; 2],
    und: [u32; 2],
    pc: u32,
    cpsr: StatusReg,
    spsr: [StatusReg; 5],
}

impl RegValues {
    pub fn new() -> RegValues {
        RegValues {
            usr: [0; 16],
            fiq: [0; 7],
            abt: [0; 2],
            svc: [0; 2],
            irq: [0; 2],
            und: [0; 2],
            pc: 0,
            cpsr: StatusReg::reset(),
            spsr: [StatusReg::empty(); 5],
        }
    }

    pub fn get_reg(&self, reg: Reg) -> u32 {
        let mode = self.cpsr.get_mode();
        use Reg::*;
        match reg {
            R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 => self.usr[reg as usize],
            R8 | R9 | R10 | R11 | R12 => match mode {
                Mode::FIQ => self.fiq[reg as usize - 8],
                _ => self.usr[reg as usize],
            },
            R13 | R14 => match mode {
                Mode::FIQ => self.fiq[reg as usize - 8],
                Mode::SVC => self.svc[reg as usize - 13],
                Mode::ABT => self.abt[reg as usize - 13],
                Mode::IRQ => self.irq[reg as usize - 13],
                Mode::UND => self.und[reg as usize - 13],
                _ => self.usr[reg as usize],
            },
            R15 => self.usr[15],
            CPSR => self.cpsr.bits,
            SPSR => match mode {
                Mode::FIQ => self.spsr[0].bits(),
                Mode::SVC => self.spsr[1].bits(),
                Mode::ABT => self.spsr[2].bits(),
                Mode::IRQ => self.spsr[3].bits(),
                Mode::UND => self.spsr[4].bits(),
                _ => panic!("No SPSR for SYS and USR"),
            },
        }
    }

    pub fn set_reg(&mut self, reg: Reg, value: u32) {
        let mode = self.cpsr.get_mode();
        use Reg::*;
        match reg {
            R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 => self.usr[reg as usize] = value,
            R8 | R9 | R10 | R11 | R12 => match mode {
                Mode::FIQ => self.fiq[reg as usize - 8] = value,
                _ => self.usr[reg as usize] = value,
            },
            R13 | R14 => match mode {
                Mode::FIQ => self.fiq[reg as usize - 8] = value,
                Mode::SVC => self.svc[reg as usize - 13] = value,
                Mode::ABT => self.abt[reg as usize - 13] = value,
                Mode::IRQ => self.irq[reg as usize - 13] = value,
                Mode::UND => self.und[reg as usize - 13] = value,
                _ => self.usr[reg as usize] = value,
            },
            R15 => self.usr[15] = value,
            CPSR => self.cpsr.bits = value,
            SPSR => match mode {
                Mode::FIQ => self.spsr[0] = StatusReg::from_bits(value).unwrap(),
                Mode::SVC => self.spsr[1] = StatusReg::from_bits(value).unwrap(),
                Mode::ABT => self.spsr[2] = StatusReg::from_bits(value).unwrap(),
                Mode::IRQ => self.spsr[3] = StatusReg::from_bits(value).unwrap(),
                Mode::UND => self.spsr[4] = StatusReg::from_bits(value).unwrap(),
                _ => panic!("No SPSR for SYS and USR"),
            },
        }
    }

    pub fn get_pc(&self) -> u32 {
        self.usr[15]
    }

    pub fn set_pc(&mut self, value: u32) {
        self.usr[15] = value;
    }

    pub fn get_n(&self) -> bool { self.cpsr.contains(StatusReg::N) }
    pub fn get_z(&self) -> bool { self.cpsr.contains(StatusReg::Z) }
    pub fn get_c(&self) -> bool { self.cpsr.contains(StatusReg::C) }
    pub fn get_v(&self) -> bool { self.cpsr.contains(StatusReg::V) }
    pub fn get_i(&self) -> bool { self.cpsr.contains(StatusReg::I) }
    pub fn get_f(&self) -> bool { self.cpsr.contains(StatusReg::F) }
    pub fn get_t(&self) -> bool { self.cpsr.contains(StatusReg::T) }
    pub fn set_n(&mut self, value: bool) { self.cpsr.set(StatusReg::N, value) }
    pub fn set_z(&mut self, value: bool) { self.cpsr.set(StatusReg::Z, value) }
    pub fn set_c(&mut self, value: bool) { self.cpsr.set(StatusReg::C, value) }
    pub fn set_v(&mut self, value: bool) { self.cpsr.set(StatusReg::V, value) }
    pub fn set_i(&mut self, value: bool) { self.cpsr.set(StatusReg::I, value) }
    pub fn set_f(&mut self, value: bool) { self.cpsr.set(StatusReg::F, value) }
    pub fn set_t(&mut self, value: bool) { self.cpsr.set(StatusReg::T, value) }
}
