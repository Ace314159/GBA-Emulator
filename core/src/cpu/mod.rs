#[cfg(test)]
#[macro_use]
mod tests;

mod arm;
mod thumb;
mod registers;

use crate::mmu::IMMU;
use registers::RegValues;

pub struct CPU {
    regs: RegValues,
    instr_buffer: [u32; 2],
    p: bool,
}

impl CPU {
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

    pub(self) fn shift(&mut self, shift_type: u32, operand: u32, shift: u32, immediate: bool, change_status: bool) -> u32 {
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
        } else {
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
}
