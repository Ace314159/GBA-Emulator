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

        if self.should_exec((instr >> 28) & 0xF) {
            if instr & 0b1111_1111_1111_1111_1111_1111_0000 == 0b0001_0010_1111_1111_1111_0001_0000 {
                self.branch_and_exchange(mmu, instr);
            } else if instr & 0b1111_1100_0000_0000_0000_1111_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.mul_mula(mmu, instr);
            } else if instr & 0b1111_1000_0000_0000_0000_1111_0000 == 0b0000_1000_0000_0000_0000_1001_0000 {
                self.mul_long(mmu, instr);
            } else if instr & 0b1111_1000_0000_0000_1111_1111_0000 == 0b0001_0000_0000_0000_0000_1001_0000 {
                self.single_data_swap(mmu, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_1001_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.halfword_and_signed_data_transfer(mmu, instr);
            } else if instr & 0b1101_1001_0000_0000_0000_0000_0000 == 0b0001_0000_0000_0000_0000_0000_0000 {
                self.psr_transfer(mmu, instr);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0000_0000_0000_0000_0000_0000_0000 {
                self.data_proc(mmu, instr);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0100_0000_0000_0000_0000_0000_0000 {
                self.single_data_transfer(mmu, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1000_0000_0000_0000_0000_0000_0000 {
                self.block_data_transfer(mmu, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1010_0000_0000_0000_0000_0000_0000 {
                self.branch_branch_with_link(mmu, instr);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1111_0000_0000_0000_0000_0000_0000 {
                self.arm_software_interrupt(mmu, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1100_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(mmu, instr);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1110_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(mmu, instr);
            } else {
                assert_eq!(instr & 0b1110_0000_0000_0000_0000_0001_0000, 0b1110_0000_0000_0000_0000_0001_0000);
                self.undefined_instr(mmu, instr);
            }
        } else { mmu.inc_clock(Cycle::N, self.regs.pc & !0x3, 2) }
    }

    // ARM.3: Branch and Exchange (BX)
    fn branch_and_exchange<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        self.regs.pc = self.regs.get_reg_i(instr & 0xF);
        if self.regs.pc & 0x1 != 0 {
            self.regs.pc -= 1;
            self.regs.set_t(true);
            self.fill_thumb_instr_buffer(mmu);
        } else { self.fill_arm_instr_buffer(mmu) }
    }

    // ARM.4: Branch and Branch with Link (B, BL)
    fn branch_branch_with_link<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        let opcode = (instr >> 24) & 0x1;
        let offset = instr & 0xFF_FFFF;
        let offset = if (offset >> 23) == 1 { 0xFF00_0000 | offset } else { offset };

        mmu.inc_clock(Cycle::N, self.regs.pc & !0x3, 2);
        if opcode == 1 { self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(4)) } // Branch with Link
        self.regs.pc = self.regs.pc.wrapping_add(offset << 2);
        self.fill_arm_instr_buffer(mmu);
    }

    // ARM.5: Data Processing
    fn data_proc<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        let change_status = (instr >> 20) & 0x1 != 0;
        let immediate_op2 = (instr >> 25) & 0x1 != 0;
        let mut temp_inc_pc = false;
        let opcode = (instr >> 21) & 0xF;
        let dest_reg = (instr >> 12) & 0xF;
        let (change_status, special_change_status) = if dest_reg == 15 && change_status {
            (false, true) } else { (change_status, false)
        };
        let op2 = if immediate_op2 {
            let shift = (instr >> 8) & 0xF;
            let operand = instr & 0xFF;
            if (opcode < 0x5 || opcode > 0x7) && shift != 0 {
                self.shift(mmu, 3, operand, shift * 2, true, change_status)
            } else { operand.rotate_right(shift * 2) }
        } else {
            let shift_by_reg = (instr >> 4) & 0x1 != 0;
            let shift = if shift_by_reg {
                assert_eq!((instr >> 7) & 0x1, 0);
                self.regs.pc = self.regs.pc.wrapping_add(4); // Temp inc
                temp_inc_pc = true;
                self.regs.get_reg_i((instr >> 8) & 0xF) & 0xFF
            } else {
                (instr >> 7) & 0x1F
            };
            let shift_type = (instr >> 5) & 0x3;
            let op2 = self.regs.get_reg_i(instr & 0xF);
            self.shift(mmu, shift_type, op2, shift, !shift_by_reg,
                change_status && (opcode < 0x5 || opcode > 0x7))
        };
        let op1 = self.regs.get_reg_i((instr >> 16) & 0xF);
        let result = match opcode {
            0x0 | 0x8 => op1 & op2, // AND and TST
            0x1 | 0x9 => op1 ^ op2, // EOR and TEQ
            0x2 | 0xA => self.sub(op1, op2, change_status), // SUB and CMP
            0x3 => self.sub(op2, op1, change_status), // RSB
            0x4 | 0xB => self.add(op1, op2, change_status), // ADD and CMN
            0x5 => self.adc(op1, op2, change_status), // ADC
            0x6 => self.sbc(op1, op2, change_status), // SBC
            0x7 => self.sbc(op2, op1, change_status), // RSC
            0xC => op1 | op2, // ORR
            0xD => op2, // MOV
            0xE => op1 & !op2, // BIC
            0xF => !op2, // MVN
            _ => panic!("Invalid opcode!"),
        };
        if change_status {
            self.regs.set_z(result == 0);
            self.regs.set_n(result & 0x8000_0000 != 0);
        } else if special_change_status { self.regs.set_reg(Reg::CPSR, self.regs.get_reg(Reg::SPSR)) }
        else { assert_eq!(opcode & 0xC != 0x8, true) }
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
    fn psr_transfer<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
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
    fn mul_mula<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 22 & 0x3F, 0b000000);
        let accumulate = instr >> 21 & 0x1 != 0;
        let change_status = instr >> 20 & 0x1 != 0;
        let dest_reg = instr >> 16 & 0xF;
        let op1_reg = instr >> 12 & 0xF;
        let op1 = self.regs.get_reg_i(op1_reg);
        let op2 = self.regs.get_reg_i(instr >> 8 & 0xF);
        assert_eq!(instr >> 4 & 0xF, 0b1001);
        let op3 = self.regs.get_reg_i(instr & 0xF);
        
        self.inc_mul_clocks(mmu, op2, true);
        let result = if accumulate {
            mmu.inc_clock(Cycle::I, 0, 0);
            op2.wrapping_mul(op3).wrapping_add(op1)
        } else {
            assert_eq!(op1_reg, 0);
            op2.wrapping_mul(op3)
        };
        if change_status {
            self.regs.set_n(result & 0x8000_0000 != 0);
            self.regs.set_z(result == 0);
        }
        self.regs.set_reg_i(dest_reg, result);

        mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2);
    }

    // ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL)
    fn mul_long<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 23 & 0x1F, 0b00001);
        let signed = instr >> 22 & 0x1 != 0;
        let accumulate = instr >> 21 & 0x1 != 0;
        let change_status = instr >> 20 & 0x1 != 0;
        let src_dest_reg_high = instr >> 16 & 0xF;
        let src_dest_reg_low = instr >> 12 & 0xF;
        let op1 = self.regs.get_reg_i(instr >> 8 & 0xF);
        assert_eq!(instr >> 4 & 0xF, 0b1001);
        let op2 = self.regs.get_reg_i(instr & 0xF);

        self.inc_mul_clocks(mmu, op1 as u32, signed);
        mmu.inc_clock(Cycle::I, 0, 0);
        let result = if signed { (op1 as i32 as u64).wrapping_mul(op2 as i32 as u64) }
        else { (op1 as u64) * (op2 as u64) }.wrapping_add(
        if accumulate {
            mmu.inc_clock(Cycle::I, 0, 0);
            (self.regs.get_reg_i(src_dest_reg_high) as u64) << 32 |
            self.regs.get_reg_i(src_dest_reg_low) as u64
        } else { 0 });
        if change_status {
            self.regs.set_n(result & 0x8000_0000_0000_0000 != 0);
            self.regs.set_z(result == 0);
        }
        self.regs.set_reg_i(src_dest_reg_low, (result >> 0) as u32);
        self.regs.set_reg_i(src_dest_reg_high, (result >> 32) as u32);
    }

    // ARM.9: Single Data Transfer (LDR, STR)
    fn single_data_transfer<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 26 & 0b11, 0b01);
        let shifted_reg_offset = instr >> 25 & 0x1 != 0;
        let pre_offset = instr >> 24 & 0x1 != 0;
        let transfer_byte = instr >> 22 & 0x1 != 0;
        let mut write_back = instr >> 21 & 0x1 != 0 || !pre_offset;
        let add_offset = instr >> 23 & 0x1 != 0;
        let load = instr >> 20 & 0x1 != 0;
        let base_reg = instr >> 16 & 0xF;
        let base = self.regs.get_reg_i(base_reg);
        let src_dest_reg = instr >> 12 & 0xF;
        mmu.inc_clock(Cycle::N, self.regs.pc, 2);

        let offset = if shifted_reg_offset {
            let shift = instr >> 7 & 0x1F;
            let shift_type = instr >> 5 & 0x3;
            assert_eq!(instr >> 4 & 0x1, 0);
            let offset_reg = instr & 0xF;
            assert_ne!(offset_reg, 15);
            let operand = self.regs.get_reg_i(offset_reg);
            self.shift(mmu, shift_type, operand, shift, true, false)
        } else {
            instr & 0xFFF
        };

        let mut exec = |addr| if load {
            mmu.inc_clock(Cycle::I, 0, 0);
            self.regs.set_reg_i(src_dest_reg, if transfer_byte {
                mmu.read8(addr) as u32
            } else {
                mmu.read32(addr & !0x3).rotate_right((addr & 0x3) * 8)
            });
            if src_dest_reg == base_reg { write_back = false }
            if src_dest_reg == 15 {
                mmu.inc_clock(Cycle::N, self.regs.pc.wrapping_add(4), 2);
                self.fill_arm_instr_buffer(mmu);
            } else { mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2); }
        } else {
            let addr = addr & !0x3;
            let value = self.regs.get_reg_i(src_dest_reg);
            let value = if src_dest_reg == 15 { value.wrapping_add(4) } else { value };
            mmu.inc_clock(Cycle::N, addr, 2);
            if transfer_byte { mmu.write8(addr, value as u8) } else { mmu.write32(addr, value) }
        };
        let offset_applied = if add_offset { base.wrapping_add(offset) } else { base.wrapping_sub(offset) };
        if pre_offset {
            exec(offset_applied);
            if write_back { self.regs.set_reg_i(base_reg, offset_applied) }
        } else {
            // TOOD: Take into account privilege of access
            let force_non_privileged_access = instr >> 21 & 0x1 != 0;
            assert_eq!(force_non_privileged_access, false);
            // Write back is not done if src_reg == base_reg
            exec(base);
            if write_back { self.regs.set_reg_i(base_reg, offset_applied) }
        }
    }

    // ARM.10: Halfword and Signed Data Transfer (STRH,LDRH,LDRSB,LDRSH)
    fn halfword_and_signed_data_transfer<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 25 & 0x7, 0b000);
        let pre_offset = instr >> 24 & 0x1 != 0;
        let add_offset = instr >> 23 & 0x1 != 0;
        let immediate_offset = instr >> 22 & 0x1 != 0;
        let mut write_back = instr >> 21 & 0x1 != 0 || !pre_offset;
        let load = instr >> 20 & 0x1 != 0;
        let base_reg = instr >> 16 & 0xF;
        let base = self.regs.get_reg_i(base_reg);
        let src_dest_reg = instr >> 12 & 0xF;
        let offset_hi = instr >> 8 & 0xF;
        assert_eq!(instr >> 7 & 0x1, 1);
        let opcode = instr >> 5 & 0x3;
        assert_eq!(instr >> 4 & 0x1, 1);
        let offset_low = instr & 0xF;
        
        let offset = if immediate_offset { offset_hi << 4 | offset_low }
        else {
            assert_eq!(offset_hi, 0);
            self.regs.get_reg_i(offset_low)
        };
        
        let mut exec = |addr| if load {
            mmu.inc_clock(Cycle::I, 0, 0);
            self.regs.set_reg_i(src_dest_reg, match opcode {
                1 => (mmu.read16(addr & !0x1) as u32).rotate_right((addr & 0x1) * 8),
                2 => mmu.read8(addr) as i8 as u32,
                3 if addr & 0x1 == 1 => mmu.read8(addr) as i8 as u32,
                3 => mmu.read16(addr) as i16 as u32,
                _ => panic!("Invalid opcode!"),
            });
            if src_dest_reg == base_reg { write_back = false }
            if src_dest_reg == 15 {
                mmu.inc_clock(Cycle::N, self.regs.pc.wrapping_add(4), 2);
                self.fill_arm_instr_buffer(mmu);
            } else { mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2); }
        } else {
            assert_eq!(opcode, 1);
            let addr = addr & !0x1;
            let value = self.regs.get_reg_i(src_dest_reg);
            mmu.inc_clock(Cycle::N, addr, 1);
            mmu.write16(addr, value as u16);
        };
        let offset_applied = if add_offset { base.wrapping_add(offset) } else { base.wrapping_sub(offset) };
        if pre_offset {
            exec(offset_applied);
            if write_back { self.regs.set_reg_i(base_reg, offset_applied) }
        } else {
            exec(base);
            assert_eq!(instr >> 24 & 0x1 != 0, false);
            // Write back is not done if src_reg == base_reg
            if write_back { self.regs.set_reg_i(base_reg, offset_applied) }
        }
    }

    // ARM.11: Block Data Transfer (LDM,STM)
    fn block_data_transfer<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 25 & 0x7, 0b100);
        let pre_offset = instr >> 24 & 0x1 != 0;
        let add_offset = instr >> 23 & 0x1 != 0;
        let psr_force_usr = instr >> 22 & 0x1 != 0;
        let write_back = instr >> 21 & 0x1 != 0;
        let load = instr >> 20 & 0x1 != 0;
        let base_reg = instr >> 16 & 0xF;
        assert_ne!(base_reg, 0xF);
        let mut base = self.regs.get_reg_i(base_reg) & !0x3;
        let mut r_list = (instr & 0xFFFF) as u16;
        let actual_mode = self.regs.get_mode();
        if psr_force_usr && !(load && r_list & 0x80 != 0) { self.regs.set_mode(Mode::USR) }

        mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        let apply_offset = |base| if add_offset { base + 4 } else { base - 4 };
        let mut calc_addr = || if pre_offset { base = apply_offset(base); (base, base) }
        else { let old_base = base; base = apply_offset(base); (old_base, base) };
        let mut loaded_pc = false;
        let mut exec = |reg, last_access| {
            let (addr, new_base) = calc_addr();
            if load {
                self.regs.set_reg_i(reg, mmu.read32(addr));
                if write_back { self.regs.set_reg_i(base_reg, new_base) }
                if reg == 15 {
                    if psr_force_usr { self.regs.restore_cpsr() }
                    loaded_pc = true;
                    self.fill_arm_instr_buffer(mmu);
                }
                if last_access { mmu.inc_clock(Cycle::I, 0, 0) }
                else { mmu.inc_clock(Cycle::S, addr, 2) }
            } else {
                mmu.write32(addr, self.regs.get_reg_i(reg));
                if last_access { mmu.inc_clock(Cycle::N, addr, 2) }
                else { mmu.inc_clock(Cycle::S, addr, 2) }
                if write_back { self.regs.set_reg_i(base_reg, new_base) }
            };
        };
        if add_offset {
            let mut reg = 0;
            while r_list != 0x1 {
                if r_list & 0x1 != 0 {
                    exec(reg, false);
                }
                reg += 1;
                r_list >>= 1;
            }
            exec(reg, true);
        } else {
            let mut reg = 15;
            while r_list != 0x8000 {
                if r_list & 0x8000 != 0 {
                    exec(reg, false);
                }
                reg -= 1;
                r_list <<= 1;
            }
            exec(reg, true);
        }

        self.regs.set_mode(actual_mode);
        if loaded_pc { mmu.inc_clock(Cycle::N, self.regs.pc.wrapping_add(4), 2) }
        else if load { mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2) }
    }

    // ARM.12: Single Data Swap (SWP)
    fn single_data_swap<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 23 & 0x1F, 0b00010);
        let byte = instr >> 22 & 0x1 != 0;
        assert_eq!(instr >> 20 & 0x3, 0b00);
        let base = self.regs.get_reg_i(instr >> 16 & 0xF);
        let dest_reg = instr >> 12 & 0xF;
        assert_eq!(instr >> 4 & 0xFF, 0b00001001);
        let src_reg = instr & 0xF;
        let src = self.regs.get_reg_i(src_reg);

        mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        let (value, access_width) = if byte {
            let value = mmu.read8(base) as u32;
            mmu.write8(base, src as u8);
            (value, 0)
        } else {
            let value = mmu.read32(base & !0x3).rotate_right((base & 0x3) * 8);
            mmu.write32(base & !0x3, src);
            (value, 2)
        };
        self.regs.set_reg_i(dest_reg, value);
        mmu.inc_clock(Cycle::N, base, access_width);

        mmu.inc_clock(Cycle::I, 0, 0);
        mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2);
    }

    // ARM.13: Software Interrupt (SWI)
    fn arm_software_interrupt<M>(&mut self, mmu: &mut M, instr: u32) where M: IMMU {
        assert_eq!(instr >> 24 & 0xF, 0b1111);
        mmu.inc_clock(Cycle::N, self.regs.pc, 2);
        let cpsr = self.regs.get_reg(Reg::CPSR);
        self.regs.set_mode(Mode::SVC);
        self.regs.set_reg(Reg::SPSR, cpsr);
        self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(4));
        self.regs.set_i(true);
        self.regs.pc = 0x8;
        self.fill_arm_instr_buffer(mmu);
    }

    // ARM.14: Coprocessor Data Operations (CDP)
    // ARM.15: Coprocessor Data Transfers (LDC,STC)
    // ARM.16: Coprocessor Register Transfers (MRC, MCR)
    fn coprocessor<M>(&mut self, _mmu: &mut M, _instr: u32) where M: IMMU {
        unimplemented!("Coprocessor not implemented!");
    }

    // ARM.17: Undefined Instruction
    fn undefined_instr<M>(&mut self, _mmu: &mut M, _instr: u32) where M: IMMU {
        unimplemented!("ARM.17: Undefined Instruction not implemented!");
    }
}
