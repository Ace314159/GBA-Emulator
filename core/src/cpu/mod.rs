#[cfg(test)]
#[macro_use]
mod tests;

mod arm;
mod thumb;
mod registers;

use crate::mmu::{IMMU, Cycle};
use registers::RegValues;

pub struct CPU {
    regs: RegValues,
    instr_buffer: [u32; 2],
    p: bool,
}

impl CPU {
    pub fn no_bios<M>(mmu: &mut M) -> CPU where M: IMMU {
        let mut cpu = CPU {
            regs: RegValues::no_bios(),
            instr_buffer: [0; 2],
            p: false,
        };
        cpu.fill_arm_instr_buffer(mmu);
        cpu
    }

    #[cfg(test)]
    pub fn new<M>(mmu: &mut M) -> CPU where M: IMMU {
        let mut cpu = CPU {
            regs: RegValues::new(),
            instr_buffer: [0; 2],
            p: true,
        };
        cpu.fill_arm_instr_buffer(mmu);
        cpu
    }

    pub fn emulate_instr<M>(&mut self, mmu: &mut M) where M: IMMU {
        if self.regs.get_t() { self.emulate_thumb_instr(mmu) }
        else { self.emulate_arm_instr(mmu) }
    }

    pub(self) fn should_exec(&self, condition: u32) -> bool {
        match condition {
            0x0 => self.regs.get_z(),
            0x1 => !self.regs.get_z(),
            0x2 => self.regs.get_c(),
            0x3 => !self.regs.get_c(),
            0x4 => self.regs.get_n(),
            0x5 => !self.regs.get_n(),
            0x6 => self.regs.get_v(),
            0x7 => !self.regs.get_v(),
            0x8 => self.regs.get_c() && !self.regs.get_z(),
            0x9 => !self.regs.get_c() | self.regs.get_z(),
            0xA => self.regs.get_n() == self.regs.get_v(),
            0xB => self.regs.get_n() != self.regs.get_v(),
            0xC => !self.regs.get_z() && self.regs.get_n() == self.regs.get_v(),
            0xD => self.regs.get_z() || self.regs.get_n() != self.regs.get_v(),
            0xE => true,
            0xF => false,
            _ => panic!("Unexpected condition!"),
        }
    }

    pub(self) fn shift<M>(&mut self, mmu: &mut M, shift_type: u32, operand: u32, shift: u32,
        immediate: bool, change_status: bool) -> u32 where M: IMMU{
        if immediate && shift == 0 {
            match shift_type {
                // LSL #0
                0 => operand,
                // LSR #32
                1 => {
                    if change_status { self.regs.set_c(operand >> 31 != 0) }
                    0
                },
                // ASR #32
                2 => {
                    let bit = operand >> 31 != 0;
                    if change_status { self.regs.set_c(bit); }
                    if bit { 0xFFFF_FFFF } else { 0 } },
                // RRX #1
                3 => {
                    let new_c = operand & 0x1 != 0;
                    let op2 = (self.regs.get_c() as u32) << 31 | operand >> 1;
                    if change_status { self.regs.set_c(new_c) }
                    op2
                },
                _ => panic!("Invalid Shift type!"),
            }
        } else if shift > 31 {
            assert_eq!(immediate, false);
            if !immediate { mmu.inc_clock(Cycle::I, 0, 0) }
            match shift_type {
                // LSL
                0 => {
                    if change_status {
                        if shift == 32 { self.regs.set_c(operand << (shift - 1) & 0x8000_0000 != 0) }
                        else { self.regs.set_c(false) }
                    }
                    0
                },
                // LSR
                1 => {
                    if change_status {
                        if shift == 32 { self.regs.set_c(operand >> (shift - 1) & 0x1 != 0)
                    } else { self.regs.set_c(false) } }
                    operand >> shift
                },
                // ASR
                2 => {
                    let c = operand & 0x8000_0000 != 0;
                    if change_status { self.regs.set_c(c) }
                    if c { 0xFFFF_FFFF } else { 0 }
                },
                // ROR
                3 => { if change_status { self.regs.set_c(operand >> (shift - 1) & 0x1 != 0); } operand.rotate_right(shift) },
                _ => panic!("Invalid Shift type!"),
            }
        } else {
            if !immediate { mmu.inc_clock(Cycle::I, 0, 0) }
            let change_status = change_status && shift != 0;
            match shift_type {
                // LSL
                0 => { if change_status { self.regs.set_c(operand << (shift - 1) & 0x8000_0000 != 0); } operand << shift },
                // LSR
                1 => { if change_status { self.regs.set_c(operand >> (shift - 1) & 0x1 != 0); } operand >> shift },
                // ASR
                2 => { if change_status { self.regs.set_c((operand as i32) >> (shift - 1) & 0x1 != 0) };
                        ((operand as i32) >> shift) as u32 },
                // ROR
                3 => { if change_status { self.regs.set_c(operand >> (shift - 1) & 0x1 != 0); } operand.rotate_right(shift) },
                _ => panic!("Invalid Shift type!"),
            }
        }
    }

    pub(self) fn add(&mut self, op1: u32, op2: u32, change_status: bool) -> u32 {
        let result = op1.overflowing_add(op2);
        if change_status {
            self.regs.set_c(result.1);
            self.regs.set_v((op1 as i32).overflowing_add(op2 as i32).1);
            self.regs.set_z(result.0 == 0);
            self.regs.set_n(result.0 & 0x8000_0000 != 0);
        }
        result.0
    }

    pub(self) fn adc(&mut self, op1: u32, op2: u32, change_status: bool) -> u32 {
        let result = op1.overflowing_add(op2);
        let result2 = result.0.overflowing_add(self.regs.get_c() as u32);
        if change_status {
            self.regs.set_c(result.1 || result2.1);
            let result = (op1 as i32).overflowing_add(op2 as i32);
            if result.1 { self.regs.set_v(true); }
            else {
                self.regs.set_v(result.0.overflowing_add(self.regs.get_c() as i32).1);
            }
            self.regs.set_z(result2.0 == 0);
            self.regs.set_n(result2.0 & 0x8000_0000 != 0);
        }
        result2.0 as u32
    }

    pub(self) fn sub(&mut self, op1: u32, op2: u32, change_status: bool) -> u32 {
        let old_c = self.regs.get_c();
        self.regs.set_c(true);
        let result = self.adc(op1, !op2, change_status); // Simulate adding op1 + !op2 + 1
        if !change_status { self.regs.set_c(old_c) }
        result
    }

    pub(self) fn sbc(&mut self, op1: u32, op2: u32, change_status: bool) -> u32 {
        self.adc(op1, !op2, change_status)
    }

    pub(self) fn inc_mul_clocks<M>(&mut self, mmu: &mut M, op1: u32, signed: bool) where M: IMMU {
        let mut mask = 0xFF_FF_FF_00;
        loop {
            mmu.inc_clock(Cycle::I, 0, 0);
            let value = op1 & mask;
            if mask == 0 || value == 0 || signed && value == mask { break }
            mask <<= 8;
        }
    }
}
