use super::CPU;
use super::IMMU;
use super::registers::{Reg, Mode};

use crate::mmu::Cycle;

#[cfg(test)]
mod tests;

impl CPU {
    pub(super) fn fill_arm_instr_buffer<M>(&mut self, mmu: &mut M) where M: IMMU {
        self.instr_buffer[0] = mmu.read32(self.regs.pc & !0x3);
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x3, 2);
        self.regs.pc = self.regs.pc.wrapping_add(4);

        self.instr_buffer[1] = mmu.read32(self.regs.pc & !0x3);
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x3, 2);
    }

    pub(super) fn emulate_arm_instr<M>(&mut self, mmu: &mut M) where M: IMMU {
        let instr = self.instr_buffer[0];
        if self.p {
            use Reg::*;
            println!("{:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} \
            {:08X} {:08X} {:08X} {:08X} cpsr: {:08X} | {:08X}",
            self.regs.get_reg(R0), self.regs.get_reg(R1), self.regs.get_reg(R2), self.regs.get_reg(R3),
            self.regs.get_reg(R4), self.regs.get_reg(R5), self.regs.get_reg(R6), self.regs.get_reg(R7),
            self.regs.get_reg(R8), self.regs.get_reg(R9), self.regs.get_reg(R10), self.regs.get_reg(R11),
            self.regs.get_reg(R12), self.regs.get_reg(R13), self.regs.get_reg(R14), self.regs.get_reg(R15),
            self.regs.get_reg(CPSR), instr);
        }
        self.instr_buffer[0] = self.instr_buffer[1];
        self.regs.pc = self.regs.pc.wrapping_add(4);
        self.instr_buffer[1] = mmu.read32(self.regs.pc & !0x3);

        if self.should_exec(instr) {
            if instr & 0b1111_1111_1111_1111_1111_1111_0000 == 0b0001_0010_1111_1111_1111_0001_0000 {
                self.branch_and_exchange(instr, mmu);
            } else if instr & 0b1111_1100_0000_0000_0000_1111_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.mul(instr, mmu);
            } else if instr & 0b1111_1000_0000_0000_0000_1111_0000 == 0b0000_1000_0000_0000_0000_1001_0000 {
                self.mul_long(instr, mmu);
            } else if instr & 0b1111_1000_0000_0000_1111_1111_0000 == 0b0001_0000_0000_0000_0000_1001_0000 {
                self.single_data_swap(instr, mmu);
            } else if instr & 0b1110_0000_0000_0000_0000_1001_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.halfword_and_signed_data_transfer(instr, mmu);
            } else if instr & 0b1101_1001_0000_0000_0000_0000_0000 == 0b0001_0000_0000_0000_0000_0000_0000 {
                self.psr_transfer(instr, mmu);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0000_0000_0000_0000_0000_0000_0000 {
                self.data_proc(instr, mmu);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0100_0000_0000_0000_0000_0000_0000 {
                self.single_data_transfer(instr, mmu);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1000_0000_0000_0000_0000_0000_0000 {
                self.block_data_transfer(instr, mmu);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1010_0000_0000_0000_0000_0000_0000 {
                self.branch_branch_with_link(instr, mmu);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1111_0000_0000_0000_0000_0000_0000 {
                self.software_interrupt(instr, mmu);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1100_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(instr, mmu);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1110_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(instr, mmu);
            } else {
                assert_eq!(instr & 0b1110_0000_0000_0000_0000_0001_0000, 0b1110_0000_0000_0000_0000_0001_0000);
                self.undefined_instr(instr, mmu);
            }
        } else { mmu.inc_clock(Cycle::N, self.regs.pc & !0x3, 2) }
    }

    fn should_exec(&self, instr: u32) -> bool {
        match (instr >> 28) & 0xF {
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
            _ => panic!("Unexpected instruction condition"),
        }
    }

    // ARM.3: Branch and Exchange (BX)
    fn branch_and_exchange<M>(&mut self, instr: u32, mmu: &mut M) where M: IMMU {
        mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        self.regs.pc = self.regs.get_reg_i(instr & 0xF);
        if self.regs.pc & 0x1 != 0 {
            self.regs.pc -= 1;
            self.regs.set_t(true);
            self.fill_thumb_instr_buffer(mmu);
        } else { self.fill_arm_instr_buffer(mmu) }
    }

    // ARM.4: Branch and Branch with Link (B, BL)
    fn branch_branch_with_link<M>(&mut self, instr: u32, mmu: &mut M) where M: IMMU {
        let opcode = (instr >> 24) & 0x1;
        let offset = instr & 0xFF_FFFF;
        let offset = if (offset >> 23) == 1 { 0xFF00_0000 | offset } else { offset };

        mmu.inc_clock(Cycle::N, self.regs.pc & !0x3, 2);
        if opcode == 1 { self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(4)) } // Branch with Link
        self.regs.pc = self.regs.pc.wrapping_add(offset << 2);
        self.fill_arm_instr_buffer(mmu);
    }

    // ARM.5: Data Processing
    fn data_proc<M>(&mut self, instr: u32, mmu: &mut M) where M: IMMU {
        let change_status = (instr >> 20) & 0x1 != 0;
        let immediate_op2 = (instr >> 25) & 0x1 != 0;
        let mut temp_inc_pc = false;
        let op2 = if immediate_op2 {
            let shift = (instr >> 8) & 0xF;
            (instr & 0xFF).rotate_right(shift * 2)
        } else {
            let shift_by_reg = (instr >> 4) & 0x1 != 0;
            let shift = if shift_by_reg {
                mmu.inc_clock(Cycle::I, 0, 0);
                assert_eq!((instr >> 7) & 0x1, 0);
                self.regs.pc = self.regs.pc.wrapping_add(4); // Temp inc
                temp_inc_pc = true;
                self.regs.get_reg_i((instr >> 8) & 0xF) & 0xFF
            } else {
                (instr >> 7) & 0x1F
            };
            let shift_type = (instr >> 5) & 0x3;
            let op2 = self.regs.get_reg_i(instr & 0xF);
            self.shift(shift_type, op2, shift, !shift_by_reg, change_status)
        };
        let opcode = (instr >> 21) & 0xF;
        let op1 = self.regs.get_reg_i((instr >> 16) & 0xF);
        macro_rules! arithmetic { ($op1:expr, $op2:expr, $func:ident, $sub:expr, $add_c:expr) => { {
            let result = ($op1 as i32).$func($op2 as i32);
            let result2 = if $add_c { result.0.overflowing_add(self.regs.get_c() as i32) }
                            else { (result.0, false) };
            if change_status {
                self.regs.set_v(result.1 || result2.1);
                let c = $op1.$func($op2).1;
                let c = if $sub { !c } else { c };
                self.regs.set_c(c);
            }
            result2.0 as u32
        } } }
        let result = match opcode {
            0x0 | 0x8 => op1 & op2, // AND and TST
            0x1 | 0x9 => op1 ^ op2, // EOR and TEQ
            0x2 | 0xA => arithmetic!(op1, op2, overflowing_sub, true, false), // SUB and CMP
            0x3 => arithmetic!(op2, op1, overflowing_sub, false, false), // RSB
            0x4 | 0xB => arithmetic!(op1, op2, overflowing_add, false, false), // ADD and CMN
            0x5 => arithmetic!(op1, op2, overflowing_add, false, true), // ADC
            0x6 => arithmetic!(op1, !op2, overflowing_add, true, true), // SBC
            0x7 => arithmetic!(op2, !op1, overflowing_add, true, true), // RSC
            0xC => op1 | op2, // ORR
            0xD => op2, // MOV
            0xE => op1 & !op2, // BIC
            0xF => !op2, // MVN
            _ => panic!("Invalid opcode!"),
        };
        let dest_reg = (instr >> 12) & 0xF;
        if change_status {
            self.regs.set_z(result == 0);
            self.regs.set_n(result & 0x8000_0000 != 0);
        } else { assert_eq!(opcode & 0xC != 0x8, true) }
        if opcode & 0xC != 0x8 {
            self.regs.set_reg_i(dest_reg, result);
            if dest_reg == 15 { self.fill_arm_instr_buffer(mmu); }
        }
        if dest_reg == 15 && opcode & 0xC != 0x8 {
            mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        } else {
            mmu.inc_clock(Cycle::S, self.regs.pc, 2);
            if temp_inc_pc { self.regs.pc = self.regs.pc.wrapping_sub(4) } // Dec after temp inc
        }
    }

    // ARM.6: PSR Transfer (MRS, MSR)
    fn psr_transfer<M>(&mut self, instr: u32, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 26 & 0b11, 0b00);
        let immediate_operand = (instr >> 25) & 0x1 != 0;
        assert_eq!(instr >> 23 & 0b11, 0b10);
        let status_reg = if instr >> 22 & 0x1 != 0 { Reg::SPSR } else { Reg::CPSR };
        let msr = instr >> 21 & 0x1 != 0;
        assert_eq!(instr >> 20 & 0b1, 0b0);
        mmu.inc_clock(Cycle::S, self.regs.pc, 2);

        if msr {
            let mut mask = 0u32;
            if instr >> 19 & 0x1 != 0 { mask |= 0xFF000000 } // Flags
            if instr >> 18 & 0x1 != 0 { mask |= 0x00FF0000 } // Status
            if instr >> 17 & 0x1 != 0 { mask |= 0x0000FF00 } // Extension
            if self.regs.get_mode() != Mode::USR && instr >> 16 & 0x1 != 0 { mask |= 0x000000FF } // Control
            assert_eq!(instr >> 12 & 0xF, 0xF);
            let operand = if immediate_operand {
                let shift = instr >> 8 & 0xF;
                (instr & 0xFF).rotate_right(shift * 2)
            } else {
                assert_eq!(instr >> 4 & 0xFF, 0);
                self.regs.get_reg_i(instr & 0xF)
            };
            let value = self.regs.get_reg(status_reg) & !mask | operand & mask;
            self.regs.set_reg(status_reg, value);
        } else {
            assert_eq!(immediate_operand, false);
            self.regs.set_reg_i(instr >> 12 & 0xF, self.regs.get_reg(status_reg));
            assert_eq!(instr & 0xFFF, 0);
        }
    }
    
    // ARM.7: Multiply and Multiply-Accumulate (MUL, MLA)
    fn mul<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.7: Multiply and Multiply-Accumulate (MUL, MLA) not implemented!");
    }

    // ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL)
    fn mul_long<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL) not implemented!");
    }

    // ARM.9: Single Data Transfer (LDR, STR)
    fn single_data_transfer<M>(&mut self, instr: u32, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 26 & 0b11, 0b01);
        let immediate_offset = instr >> 25 & 0x1 != 0;
        let pre_offset = instr >> 24 & 0x1 != 0;
        let transfer_byte = instr >> 22 & 0x1 != 0;
        let add_offset = instr >> 23 & 0x1 != 0;
        let load = instr >> 20 & 0x1 != 0;
        let base_reg = instr >> 16 & 0xF;
        let base = self.regs.get_reg_i(base_reg);
        let src_dest_reg = instr >> 12 & 0xF;
        mmu.inc_clock(Cycle::N, self.regs.pc, 2);

        let offset = if immediate_offset {
            let shift = instr >> 0x1F;
            let shift_type = instr >> 5 & 0x3;
            assert_eq!(instr >> 4 & 0x1, 0);
            let operand = self.regs.get_reg_i(instr & 0x7);
            self.shift(shift_type, operand, shift, false, false)
        } else {
            instr & 0xFFF
        };

        let mut exec = |addr| if load {
            mmu.inc_clock(Cycle::I, 0, 0);
            self.regs.set_reg_i(src_dest_reg, if transfer_byte {
                mmu.read8(addr) as u32
            } else { mmu.read32(addr).rotate_right((addr & 0b11) * 8) });
            if src_dest_reg == 15 {
                mmu.inc_clock(Cycle::N, self.regs.pc.wrapping_add(4), 2);
                self.fill_arm_instr_buffer(mmu);
            } else { mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2); }
        } else {
            let value = self.regs.get_reg_i(src_dest_reg);
            mmu.inc_clock(Cycle::N, addr, 2);
            if transfer_byte { mmu.write8(addr, value as u8) } else { mmu.write32(addr, value) }
        };
        let offset_applied = if add_offset { base.wrapping_add(offset) } else { base.wrapping_sub(offset) };
        if pre_offset {
            exec(offset_applied);
            if instr >> 21 & 0x1 != 0 { self.regs.set_reg_i(base_reg, offset_applied) }
        } else {
            // TOOD: Take into account privilege of access
            exec(base);
            self.regs.set_reg_i(base_reg, offset_applied);
        }
    }

    // ARM.10: Halfword and Signed Data Transfer (STRH,LDRH,LDRSB,LDRSH)
    fn halfword_and_signed_data_transfer<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.10: Halfword and Signed Data Transfer (STRH,LDRH,LDRSB,LDRSH) not implemented!");
    }

    // ARM.11: Block Data Transfer (LDM,STM)
    fn block_data_transfer<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.11: Block Data Transfer (LDM,STM) not implemented!");
    }

    // ARM.12: Single Data Swap (SWP)
    fn single_data_swap<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.12: Single Data Swap (SWP) not implemented!");
    }

    // ARM.13: Software Interrupt (SWI)
    fn software_interrupt<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.13: Software Interrupt (SWI) not implemented!");
    }

    // ARM.14: Coprocessor Data Operations (CDP)
    // ARM.15: Coprocessor Data Transfers (LDC,STC)
    // ARM.16: Coprocessor Register Transfers (MRC, MCR)
    fn coprocessor<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("Coprocessor not implemented!");
    }

    // ARM.17: Undefined Instruction
    fn undefined_instr<M>(&mut self, _instr: u32, _mmu: &mut M) where M: IMMU {
        unimplemented!("ARM.17: Undefined Instruction not implemented!");
    }
}
