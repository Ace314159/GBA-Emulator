use super::{
    CPU, IO,
    registers::{Reg, Mode}
};

use crate::io::AccessType;

impl CPU {
    pub(super) fn fill_arm_instr_buffer(&mut self, io: &mut IO) {
        self.regs.pc &= !0x3;
        self.instr_buffer[0] = self.read::<u32>(io, AccessType::S, self.regs.pc & !0x3);
        self.regs.pc = self.regs.pc.wrapping_add(4);

        self.instr_buffer[1] = self.read::<u32>(io, AccessType::S, self.regs.pc & !0x3);
    }

    pub(super) fn emulate_arm_instr(&mut self, io: &mut IO) {
        let instr = self.instr_buffer[0];
        {
            use Reg::*;
            trace!("{:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} \
            {:08X} {:08X} {:08X} {:08X} cpsr: {:08X} | {:08X}",
            self.regs.get_reg(R0), self.regs.get_reg(R1), self.regs.get_reg(R2), self.regs.get_reg(R3),
            self.regs.get_reg(R4), self.regs.get_reg(R5), self.regs.get_reg(R6), self.regs.get_reg(R7),
            self.regs.get_reg(R8), self.regs.get_reg(R9), self.regs.get_reg(R10), self.regs.get_reg(R11),
            self.regs.get_reg(R12), self.regs.get_reg(R13), self.regs.get_reg(R14), self.regs.get_reg(R15),
            self.regs.get_reg(CPSR), instr);
        }
        self.instr_buffer[0] = self.instr_buffer[1];
        self.regs.pc = self.regs.pc.wrapping_add(4);

        if self.should_exec((instr >> 28) & 0xF) {
            if instr & 0b1111_1111_1111_1111_1111_1111_0000 == 0b0001_0010_1111_1111_1111_0001_0000 {
                self.branch_and_exchange(io, instr);
            } else if instr & 0b1111_1100_0000_0000_0000_1111_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.mul_mula(io, instr);
            } else if instr & 0b1111_1000_0000_0000_0000_1111_0000 == 0b0000_1000_0000_0000_0000_1001_0000 {
                self.mul_long(io, instr);
            } else if instr & 0b1111_1000_0000_0000_1111_1111_0000 == 0b0001_0000_0000_0000_0000_1001_0000 {
                self.single_data_swap(io, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_1001_0000 == 0b0000_0000_0000_0000_0000_1001_0000 {
                self.halfword_and_signed_data_transfer(io, instr);
            } else if instr & 0b1101_1001_0000_0000_0000_0000_0000 == 0b0001_0000_0000_0000_0000_0000_0000 {
                self.psr_transfer(io, instr);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0000_0000_0000_0000_0000_0000_0000 {
                self.data_proc(io, instr);
            } else if instr & 0b1100_0000_0000_0000_0000_0000_0000 == 0b0100_0000_0000_0000_0000_0000_0000 {
                self.single_data_transfer(io, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1000_0000_0000_0000_0000_0000_0000 {
                self.block_data_transfer(io, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1010_0000_0000_0000_0000_0000_0000 {
                self.branch_branch_with_link(io, instr);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1111_0000_0000_0000_0000_0000_0000 {
                self.arm_software_interrupt(io, instr);
            } else if instr & 0b1110_0000_0000_0000_0000_0000_0000 == 0b1100_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(io, instr);
            } else if instr & 0b1111_0000_0000_0000_0000_0000_0000 == 0b1110_0000_0000_0000_0000_0000_0000 {
                self.coprocessor(io, instr);
            } else {
                assert_eq!(instr & 0b1110_0000_0000_0000_0000_0001_0000, 0b1110_0000_0000_0000_0000_0001_0000);
                self.undefined_instr(io, instr);
            }
        } else {
            self.instruction_prefetch::<u32>(io, AccessType::S);
        }
    }

    // ARM.3: Branch and Exchange (BX)
    fn branch_and_exchange(&mut self, io: &mut IO, instr: u32) {
        self.instruction_prefetch::<u32>(io, AccessType::N);
        self.regs.pc = self.regs.get_reg_i(instr & 0xF);
        if self.regs.pc & 0x1 != 0 {
            self.regs.pc -= 1;
            self.regs.set_t(true);
            self.fill_thumb_instr_buffer(io);
        } else { self.fill_arm_instr_buffer(io) }
    }

    // ARM.4: Branch and Branch with Link (B, BL)
    fn branch_branch_with_link(&mut self, io: &mut IO, instr: u32) {
        let opcode = (instr >> 24) & 0x1;
        let offset = instr & 0xFF_FFFF;
        let offset = if (offset >> 23) == 1 { 0xFF00_0000 | offset } else { offset };

        self.instruction_prefetch::<u32>(io, AccessType::N);
        if opcode == 1 { self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(4)) } // Branch with Link
        self.regs.pc = self.regs.pc.wrapping_add(offset << 2);
        self.fill_arm_instr_buffer(io);
    }

    // ARM.5: Data Processing
    fn data_proc(&mut self, io: &mut IO, instr: u32) {
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
                self.shift(io, 3, operand, shift * 2, true, change_status)
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
            // TODO: I Cycle occurs too early
            self.shift(io, shift_type, op2, shift, !shift_by_reg,
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
            _ => unreachable!(),
        };
        if change_status {
            self.regs.set_z(result == 0);
            self.regs.set_n(result & 0x8000_0000 != 0);
        } else if special_change_status { self.regs.set_reg(Reg::CPSR, self.regs.get_reg(Reg::SPSR)) }
        else { assert_eq!(opcode & 0xC != 0x8, true) }
        let mut clocked = false;
        if opcode & 0xC != 0x8 {
            self.regs.set_reg_i(dest_reg, result);
            if dest_reg == 15 {
                clocked = true;
                self.instruction_prefetch::<u32>(io, AccessType::N);
                if self.regs.get_t() { self.fill_thumb_instr_buffer(io) }
                else { self.fill_arm_instr_buffer(io) }
            }
        }
        if !clocked {
            if temp_inc_pc { self.regs.pc = self.regs.pc.wrapping_sub(4) } // Dec after temp inc
            self.instruction_prefetch::<u32>(io, AccessType::S);
        }
    }

    // ARM.6: PSR Transfer (MRS, MSR)
    fn psr_transfer(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 26 & 0b11, 0b00);
        let immediate_operand = (instr >> 25) & 0x1 != 0;
        assert_eq!(instr >> 23 & 0b11, 0b10);
        let status_reg = if instr >> 22 & 0x1 != 0 { Reg::SPSR } else { Reg::CPSR };
        let msr = instr >> 21 & 0x1 != 0;
        assert_eq!(instr >> 20 & 0b1, 0b0);
        self.instruction_prefetch::<u32>(io, AccessType::S);

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
    fn mul_mula(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 22 & 0x3F, 0b000000);
        let accumulate = instr >> 21 & 0x1 != 0;
        let change_status = instr >> 20 & 0x1 != 0;
        let dest_reg = instr >> 16 & 0xF;
        let op1_reg = instr >> 12 & 0xF;
        let op1 = self.regs.get_reg_i(op1_reg);
        let op2 = self.regs.get_reg_i(instr >> 8 & 0xF);
        assert_eq!(instr >> 4 & 0xF, 0b1001);
        let op3 = self.regs.get_reg_i(instr & 0xF);
        
        self.instruction_prefetch::<u32>(io, AccessType::S);
        self.inc_mul_clocks(io, op2, true);
        let result = if accumulate {
            self.internal(io);
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
    }

    // ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL)
    fn mul_long(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 23 & 0x1F, 0b00001);
        let signed = instr >> 22 & 0x1 != 0;
        let accumulate = instr >> 21 & 0x1 != 0;
        let change_status = instr >> 20 & 0x1 != 0;
        let src_dest_reg_high = instr >> 16 & 0xF;
        let src_dest_reg_low = instr >> 12 & 0xF;
        let op1 = self.regs.get_reg_i(instr >> 8 & 0xF);
        assert_eq!(instr >> 4 & 0xF, 0b1001);
        let op2 = self.regs.get_reg_i(instr & 0xF);

        self.instruction_prefetch::<u32>(io, AccessType::S);
        self.inc_mul_clocks(io, op1 as u32, signed);
        let result = if signed { (op1 as i32 as u64).wrapping_mul(op2 as i32 as u64) }
        else { (op1 as u64) * (op2 as u64) }.wrapping_add(
        if accumulate {
            self.internal(io);
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
    fn single_data_transfer(&mut self, io: &mut IO, instr: u32) {
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
        self.instruction_prefetch::<u32>(io, AccessType::N);

        let offset = if shifted_reg_offset {
            let shift = instr >> 7 & 0x1F;
            let shift_type = instr >> 5 & 0x3;
            assert_eq!(instr >> 4 & 0x1, 0);
            let offset_reg = instr & 0xF;
            assert_ne!(offset_reg, 15);
            let operand = self.regs.get_reg_i(offset_reg);
            self.shift(io, shift_type, operand, shift, true, false)
        } else {
            instr & 0xFFF
        };

        let mut exec = |addr| if load {
            let access_type = if src_dest_reg == 15 { AccessType::N } else { AccessType::S };
            let value = if transfer_byte {
                self.read::<u8>(io, access_type, addr) as u32
            } else {
                self.read::<u32>(io, access_type, addr & !0x3).rotate_right((addr & 0x3) * 8)
            };
            self.internal(io);
            self.regs.set_reg_i(src_dest_reg, value);
            if src_dest_reg == base_reg { write_back = false }
            if src_dest_reg == 15 { self.fill_arm_instr_buffer(io) }
        } else {
            let value = self.regs.get_reg_i(src_dest_reg);
            let value = if src_dest_reg == 15 { value.wrapping_add(4) } else { value };
            if transfer_byte {
                self.write::<u8>(io, AccessType::N, addr, value as u8);
            } else {
                self.write::<u32>(io, AccessType::N, addr & !0x3, value);
            }
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
    fn halfword_and_signed_data_transfer(&mut self, io: &mut IO, instr: u32) {
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
        self.instruction_prefetch::<u32>(io, AccessType::N);
        
        let offset = if immediate_offset { offset_hi << 4 | offset_low }
        else {
            assert_eq!(offset_hi, 0);
            self.regs.get_reg_i(offset_low)
        };
        
        let mut exec = |addr| if load {
            if src_dest_reg == base_reg { write_back = false }
            let access_type = if src_dest_reg == 15 { AccessType::N } else { AccessType::S };
            // TODO: Make all access 16 bit
            let value = match opcode {
                1 => (self.read::<u16>(io, access_type, addr & !0x1) as u32).rotate_right((addr & 0x1) * 8),
                2 => self.read::<u8>(io, access_type, addr) as i8 as u32,
                3 if addr & 0x1 == 1 => self.read::<u8>(io, access_type, addr) as i8 as u32,
                3 => self.read::<u16>(io, access_type, addr) as i16 as u32,
                _ => unreachable!(),
            };
            self.internal(io);
            self.regs.set_reg_i(src_dest_reg, value);
            if src_dest_reg == 15 { self.fill_arm_instr_buffer(io) }
        } else {
            assert_eq!(opcode, 1);
            let addr = addr & !0x1;
            let value = self.regs.get_reg_i(src_dest_reg);
            self.write::<u16>(io, AccessType::N, addr, value as u16);
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
    fn block_data_transfer(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 25 & 0x7, 0b100);
        let add_offset = instr >> 23 & 0x1 != 0;
        let pre_offset = (instr >> 24 & 0x1 != 0) ^ !add_offset;
        let psr_force_usr = instr >> 22 & 0x1 != 0;
        let load = instr >> 20 & 0x1 != 0;
        let write_back = instr >> 21 & 0x1 != 0;
        let base_reg = instr >> 16 & 0xF;
        assert_ne!(base_reg, 0xF);
        let base = self.regs.get_reg_i(base_reg);
        let base_offset = base & 0x3;
        let base = base - base_offset;
        let mut r_list = (instr & 0xFFFF) as u16;
        let write_back = write_back && !(load && r_list & (1 << base_reg) != 0);
        let actual_mode = self.regs.get_mode();
        if psr_force_usr && !(load && r_list & 0x80 != 0) { self.regs.set_mode(Mode::USR) }

        self.instruction_prefetch::<u32>(io, AccessType::N);
        let mut loaded_pc = false;
        let num_regs = r_list.count_ones();
        let start_addr = if add_offset { base } else { base.wrapping_sub(num_regs * 4) };
        let mut addr = start_addr;
        let final_addr = if add_offset { addr + 4 * num_regs } else { start_addr } + base_offset;
        let (final_addr, inc_amount) = if num_regs == 0 {
            (final_addr + 0x40, 0x40)
        } else { (final_addr, 4) };
        let mut calc_addr = || if pre_offset { addr += inc_amount; addr }
        else { let old_addr = addr; addr += inc_amount; old_addr };
        let mut exec = |addr, reg, last_access| if load {
            let value = self.read::<u32>(io, AccessType::S, addr);
            self.regs.set_reg_i(reg, value);
            if write_back { self.regs.set_reg_i(base_reg, final_addr) }
            if last_access { self.internal(io) }
            if reg == 15 {
                if psr_force_usr { self.regs.restore_cpsr() }
                loaded_pc = true;
                // TODO: Verify
                self.next_access_type = AccessType::N;
                self.fill_arm_instr_buffer(io);
            }
        } else {
            let value = self.regs.get_reg_i(reg);
            let access_type = if last_access { AccessType::N } else { AccessType::S };
            self.write::<u32>(io, access_type, addr, if reg == 15 { value.wrapping_add(4) } else { value });
            if write_back { self.regs.set_reg_i(base_reg, final_addr) }
        };
        if num_regs == 0 {
            exec(start_addr, 15, true);
        } else {
            let mut reg = 0;
            while r_list != 0x1 {
                if r_list & 0x1 != 0 {
                    exec(calc_addr(), reg, false);
                }
                reg += 1;
                r_list >>= 1;
            }
            exec(calc_addr(), reg, true);
        }

        self.regs.set_mode(actual_mode);
        // TODO: Test what this is for
        //if loaded_pc { io.inc_clock(Cycle::N, self.regs.pc.wrapping_add(4), 2) }
        //else if load { io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(4), 2) }
    }

    // ARM.12: Single Data Swap (SWP)
    fn single_data_swap(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 23 & 0x1F, 0b00010);
        let byte = instr >> 22 & 0x1 != 0;
        assert_eq!(instr >> 20 & 0x3, 0b00);
        let base = self.regs.get_reg_i(instr >> 16 & 0xF);
        let dest_reg = instr >> 12 & 0xF;
        assert_eq!(instr >> 4 & 0xFF, 0b00001001);
        let src_reg = instr & 0xF;
        let src = self.regs.get_reg_i(src_reg);

        self.instruction_prefetch::<u32>(io, AccessType::N);
        let value = if byte {
            let value = self.read::<u8>(io, AccessType::N, base) as u32;
            self.write::<u8>(io, AccessType::S, base, src as u8);
            value
        } else {
            let value = self.read::<u32>(io, AccessType::N, base & !0x3).rotate_right((base & 0x3) * 8);
            self.write::<u32>(io, AccessType::S, base & !0x3, src);
            value
        };
        self.regs.set_reg_i(dest_reg, value);
        self.internal(io);
    }

    // ARM.13: Software Interrupt (SWI)
    fn arm_software_interrupt(&mut self, io: &mut IO, instr: u32) {
        assert_eq!(instr >> 24 & 0xF, 0b1111);
        self.instruction_prefetch::<u32>(io, AccessType::N);
        self.regs.change_mode(Mode::SVC);
        self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(4));
        self.regs.set_i(true);
        self.regs.pc = 0x8;
        self.fill_arm_instr_buffer(io);
    }

    // ARM.14: Coprocessor Data Operations (CDP)
    // ARM.15: Coprocessor Data Transfers (LDC,STC)
    // ARM.16: Coprocessor Register Transfers (MRC, MCR)
    fn coprocessor(&mut self, _io: &mut IO, _instr: u32) {
        unimplemented!("Coprocessor not implemented!");
    }

    // ARM.17: Undefined Instruction
    fn undefined_instr(&mut self, _io: &mut IO, _instr: u32) {
        unimplemented!("ARM.17: Undefined Instruction not implemented!");
    }
}
