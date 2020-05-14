use super::CPU;
use super::IMMU;
use super::registers::Reg;

use crate::mmu::Cycle;

#[cfg(test)]
mod tests;

impl CPU {
    pub(super) fn fill_thumb_instr_buffer<M>(&mut self, mmu: &mut M) where M: IMMU {
        self.instr_buffer[0] = mmu.read16(self.regs.pc & !0x1) as u32;
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
        self.regs.pc = self.regs.pc.wrapping_add(2);

        self.instr_buffer[1] = mmu.read16(self.regs.pc & !0x1) as u32;
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
    }

    pub(super) fn emulate_thumb_instr<M>(&mut self, mmu: &mut M) where M: IMMU {
        let instr = self.instr_buffer[0] as u16;
        if self.p {
            use Reg::*;
            println!("{:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} \
            {:08X} {:08X} {:08X} {:08X} cpsr: {:08X} | {}",
            self.regs.get_reg(R0), self.regs.get_reg(R1), self.regs.get_reg(R2), self.regs.get_reg(R3),
            self.regs.get_reg(R4), self.regs.get_reg(R5), self.regs.get_reg(R6), self.regs.get_reg(R7),
            self.regs.get_reg(R8), self.regs.get_reg(R9), self.regs.get_reg(R10), self.regs.get_reg(R11),
            self.regs.get_reg(R12), self.regs.get_reg(R13), self.regs.get_reg(R14), self.regs.get_reg(R15),
            self.regs.get_reg(CPSR), if instr & 0b1111_1000_0000_0000 == 0b1111_0000_0000_0000 {
                format!("{:04X}{:04X}", instr, self.instr_buffer[1])
            } else { format!("    {:04X}", instr) });
        }
        self.instr_buffer[0] = self.instr_buffer[1];
        self.regs.pc = self.regs.pc.wrapping_add(2);
        self.instr_buffer[1] = mmu.read16(self.regs.pc & !0x1) as u32;

        if instr & 0b1111_1000_0000_0000 == 0b0001_1000_0000_0000 { self.add_sub(instr, mmu) }
        else if instr & 0b1110_0000_0000_0000 == 0b0000_0000_0000_0000 { self.move_shifted_reg(instr, mmu) }
        else if instr & 0b1110_0000_0000_0000 == 0b0010_0000_0000_0000 { self.immediate(instr, mmu) }
        else if instr & 0b1111_1100_0000_0000 == 0b0100_0000_0000_0000 { self.alu(instr, mmu) }
        else if instr & 0b1111_1100_0000_0000 == 0b0100_0100_0000_0000 { self.hi_reg_bx(instr, mmu) }
        else if instr & 0b1111_1000_0000_0000 == 0b0100_1000_0000_0000 { self.load_pc_rel(instr, mmu) }
        else if instr & 0b1111_1001_0000_0000 == 0b0101_0000_0000_0000 { self.load_store_reg_offset(instr, mmu) }
        else if instr & 0b1111_1001_0000_0000 == 0b0101_0010_0000_0000 { self.load_store_sign_ext(instr, mmu) }
        else if instr & 0b1110_0000_0000_0000 == 0b0110_0000_0000_0000 { self.load_store_imm_offset(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1000_0000_0000_0000 { self.load_store_halfword(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1001_0000_0000_0000 { self.load_store_sp_rel(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1010_0000_0000_0000 { self.get_rel_addr(instr, mmu) }
        else if instr & 0b1111_1111_0000_0000 == 0b1011_0000_0000_0000 { self.add_offset_sp(instr, mmu) }
        else if instr & 0b1111_0110_0000_0000 == 0b1011_0100_0000_0000 { self.push_pop_regs(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1100_0000_0000_0000 { self.multiple_load_store(instr, mmu) }
        else if instr & 0b1111_1111_0000_0000 == 0b1101_1111_0000_0000 { self.thumb_software_interrupt(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1101_0000_0000_0000 { self.cond_branch(instr, mmu) }
        else if instr & 0b1111_1000_0000_0000 == 0b1110_0000_0000_0000 { self.uncond_branch(instr, mmu) }
        else if instr & 0b1111_0000_0000_0000 == 0b1111_0000_0000_0000 { self.branch_with_link(instr, mmu) }
        else { panic!("Unexpected Instruction: {:08X}", instr) }
    }
    
    // THUMB.1: move shifted register
    fn move_shifted_reg<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 13, 0b000);
        let opcode = (instr >> 12 & 0x3) as u32;
        let offset = (instr >> 6 & 0x1F) as u32;
        let src = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let dest_reg = (instr & 0x7) as u32;
        assert_ne!(opcode, 0b11);
        let result = self.shift(mmu, opcode, src, offset, true, true);

        self.regs.set_n(result & 0x8000_0000 != 0);
        self.regs.set_z(result == 0);
        self.regs.set_reg_i(dest_reg, result);
        mmu.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.2: add/subtract
    fn add_sub<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 11, 0b00011);
        let sub = instr >> 9 & 0x1 != 0;
        let operand = (instr >> 6 & 0x7) as u32;
        let operand = if instr >> 10 & 0x1 != 0 { operand } else { self.regs.get_reg_i(operand) };
        let src = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let dest_reg = (instr & 0x7) as u32;

        let result = if sub {
            self.sub(src, operand, true)
        } else {
            self.add(src, operand, true)
        };
        self.regs.set_reg_i(dest_reg, result);
        mmu.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.3: move/compare/add/subtract immediate
    fn immediate<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 13, 0b001);
        let opcode = instr >> 11 & 0x3;
        let dest_reg = instr >> 8 & 0x7;
        let immediate = (instr & 0xFF) as u32;
        let op1 = self.regs.get_reg_i(dest_reg as u32);
        let result = match opcode {
            0b00 => immediate, // MOV
            0b01 => self.sub(op1, immediate, true), // CMP
            0b10 => self.add(op1, immediate, true), // ADD
            0b11 => self.sub(op1, immediate, true), // SUB
            _ => panic!("Invalid opcode!"),
        };
        self.regs.set_z(result == 0);
        self.regs.set_n(result & 0x8000_0000 != 0);

        if opcode != 0b01 { self.regs.set_reg_i(dest_reg as u32, result) }
        mmu.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.4: ALU operations
    fn alu<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 10 & 0x3F, 0b010000);
        let opcode = instr >> 6 & 0xF;
        let src = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let dest_reg = (instr & 0x7) as u32;
        let dest = self.regs.get_reg_i(dest_reg);
        let result = match opcode {
            0x0 => dest & src, // AND
            0x1 => dest ^ src, // XOR 
            0x2 => self.shift(mmu, 0, dest, src & 0xFF, false, true), // LSL
            0x3 => self.shift(mmu, 1, dest, src & 0xFF, false, true), // LSR
            0x4 => self.shift(mmu, 2, dest, src, false, true), // ASR
            0x5 => self.adc(dest, src, true), // ADC
            0x6 => self.sbc(dest, src, true), // SBC
            0x7 => self.shift(mmu, 3, dest, src & 0xFF, false, true), // ROR
            0x8 => dest & (src & 0xFF), // TST
            0x9 => self.sub(0, dest, true), // NEG
            0xA => self.sub(dest, src, true), // CMP
            0xB => self.add(dest, src, true), // CMN
            0xC => dest | src, // ORR
            0xD => self.mul(mmu, dest, src), // MUL
            0xE => dest & !src, // BIC
            0xF => !src, // MVN
            _ => panic!("Invalid opcode!"),
        };
        self.regs.set_n(result & 0x8000_0000 != 0);
        self.regs.set_z(result == 0);

        if ![0x8, 0xA, 0xB].contains(&opcode) { self.regs.set_reg_i(dest_reg, result) }
        mmu.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.5: Hi register operations/branch exchange
    fn hi_reg_bx<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 10, 0b010001);
        let opcode = instr >> 8 & 0x3;
        let dest_reg_msb = instr >> 7 & 0x1;
        let src_reg_msb = instr >> 6 & 0x1;
        let src = self.regs.get_reg_i((src_reg_msb << 3 | instr >> 3 & 0x7) as u32);
        let dest_reg = (dest_reg_msb << 3 | instr & 0x7) as u32;
        let dest = self.regs.get_reg_i(dest_reg);
        let result = match opcode {
            0b00 => self.add(dest,src, false), // ADD
            0b01 => self.sub(dest, src, true), // CMP
            0b10 => src,
            0b11 => {
                assert_eq!(dest_reg_msb, 0);
                self.regs.pc = src;
                mmu.inc_clock(Cycle::N, self.regs.pc, 1);
                if src & 0x1 != 0 { self.fill_thumb_instr_buffer(mmu) }
                else {
                    self.regs.pc = self.regs.pc & !0x2;
                    self.regs.set_t(false);
                    self.fill_arm_instr_buffer(mmu);
                }
                return
            },
            _ => panic!("Invalid Opcode!"),
        };
        if opcode & 0x1 == 0 { self.regs.set_reg_i(dest_reg, result) }
        if dest_reg == 15 {
            mmu.inc_clock(Cycle::N, self.regs.pc, 1);
            self.fill_thumb_instr_buffer(mmu);
        } else { mmu.inc_clock(Cycle::S, self.regs.pc, 1) }
    }

    // THUMB.6: load PC-relative
    fn load_pc_rel<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 11, 0b01001);
        let dest_reg = (instr >> 8 & 0x7) as u32;
        let offset = (instr & 0xFF) as u32;
        let addr = (self.regs.pc & !0x2).wrapping_add(offset * 4);
        self.regs.set_reg_i(dest_reg, mmu.read32(addr) as u32);
        mmu.inc_clock(Cycle::N, self.regs.pc, 1);
        mmu.inc_clock(Cycle::I, 0, 0);
        mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
    }

    // THUMB.7: load/store with register offset
    fn load_store_reg_offset<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12, 0b0101);
        let opcode = instr >> 10 & 0x3; 
        assert_eq!(instr >> 9 & 0x1, 0);
        let offset_reg = (instr >> 6 & 0x7) as u32;
        let base_reg = (instr >> 3 & 0x7) as u32;
        let addr = self.regs.get_reg_i(base_reg).wrapping_add(self.regs.get_reg_i(offset_reg));
        let src_dest_reg = (instr & 0x7) as u32;
        mmu.inc_clock(Cycle::N, self.regs.pc, 1);
        if opcode & 0b10 != 0 { // Load
            mmu.inc_clock(Cycle::I, 0, 0);
            mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, if opcode & 0b01 != 0 {
                mmu.read8(addr) as u32 // LDRB
            } else {
                mmu.read32(addr) // LDR
            });

        } else { // Store
            let access_width = if opcode & 0b01 != 0 { // STRB
                mmu.write8(addr, self.regs.get_reg_i(src_dest_reg) as u8);
                1
            } else { // STR
                mmu.write32(addr, self.regs.get_reg_i(src_dest_reg));
                0
            };
            mmu.inc_clock(Cycle::N, addr, access_width);
        }
    }

    // THUMB.8: load/store sign-extended byte/halfword
    fn load_store_sign_ext<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.8: load/store sign-extended byte/halfword not implemented!")
    }

    // THUMB.9: load/store with immediate offset
    fn load_store_imm_offset<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.9: load/store with immediate offset not implemented!")
    }

    // THUMB.10: load/store halfword
    fn load_store_halfword<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12, 0b1000);
        let load = instr >> 1 & 0x1 != 0;
        let offset = (instr >> 6 & 0x1F) as u32;
        let base = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let src_dest_reg = (instr & 0x7) as u32;
        let addr = base + offset * 2;

        mmu.inc_clock(Cycle::N, self.regs.pc, 1);
        if load {
            mmu.inc_clock(Cycle::I, 0, 0);
            mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, mmu.read16(addr) as u32);
        } else {
            mmu.inc_clock(Cycle::N, addr, 1);
            mmu.write16(addr, self.regs.get_reg_i(src_dest_reg) as u16);
        }
    }

    // THUMB.11: load/store SP-relative
    fn load_store_sp_rel<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12 & 0xF, 0b1001);
        let load = instr >> 11 & 0x1 != 0;
        let src_dest_reg = (instr >> 8 & 0x7) as u32;
        let offset = (instr & 0xFF) * 4;
        let addr = self.regs.get_reg(Reg::R13).wrapping_add(offset as u32);
        mmu.inc_clock(Cycle::N, self.regs.pc, 1);
        if load {
            mmu.inc_clock(Cycle::I, 0, 0);
            self.regs.set_reg_i(src_dest_reg, mmu.read32(addr));
            mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
        } else {
            mmu.inc_clock(Cycle::N, addr, 2);
            mmu.write32(addr, self.regs.get_reg_i(src_dest_reg));
        }
    }

    // THUMB.12: get relative address
    fn get_rel_addr<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.12: get relative address not implemented!")
    }

    // THUMB.13: add offset to stack pointer
    fn add_offset_sp<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 8 & 0xFF, 0b10110000);
        let sub = instr >> 7 & 0x1 != 0;
        let offset = ((instr & 0x7F) * 4) as u32;
        let sp = self.regs.get_reg(Reg::R13);
        let value = if sub { sp.wrapping_sub(offset) } else { sp.wrapping_add(offset) };
        self.regs.set_reg(Reg::R13, value);
        mmu.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.14: push/pop registers
    fn push_pop_regs<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12 & 0xF, 0b1011);
        let pop = instr >> 11 & 0x1 != 0;
        assert_eq!(instr >> 9 & 0x3, 0b10);
        let pc_lr = instr >> 8 & 0x1 != 0;
        let mut r_list = (instr & 0xFF) as u8;
        mmu.inc_clock(Cycle::N, self.regs.pc, 1);
        if pop {
            let mut reg = 0;
            while r_list != 0 {
                if r_list & 0x1 != 0 {
                    let value = self.stack_pop(mmu, r_list == 1 && !pc_lr);
                    self.regs.set_reg_i(reg, value);
                }
                reg += 1;
                r_list >>= 1;
            }
        } else {
            let mut reg = 8;
            while r_list != 0 {
                reg -= 1;
                if r_list & 0x80 != 0 {
                    self.stack_push(mmu, self.regs.get_reg_i(reg), r_list == 0x80 && !pc_lr);
                }
                r_list <<= 1;
            }
        }
        if pc_lr {
            if pop {
                mmu.inc_clock(Cycle::N, self.regs.pc.wrapping_add(2), 1);
                self.regs.pc = self.stack_pop(mmu, true) & !0x1;
                self.fill_thumb_instr_buffer(mmu);
            } else {
                self.stack_push(mmu, self.regs.get_reg(Reg::R14), true);
            }
        } else if pop {
            mmu.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
        }
    }

    // THUMB.15: multiple load/store
    fn multiple_load_store<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.15: multiple load/store not implemented!")
    }

    // THUMB.16: conditional branch
    fn cond_branch<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12, 0b1101);
        let opcode = instr >> 8 & 0xF;
        let offset = (instr & 0xFF) as i8 as u32;
        let condition = match opcode {
            0x0 => self.regs.get_z(), // BEQ
            0x1 => !self.regs.get_z(), // BNE 
            0x2 => self.regs.get_c(), // BCS
            0x3 => !self.regs.get_c(), // BCC
            0x4 => self.regs.get_n(), // BMI
            0x5 => !self.regs.get_n(), // BPL
            0x6 => self.regs.get_v(), // BVS
            0x7 => !self.regs.get_v(), // BVC
            0x8 => self.regs.get_c() && !self.regs.get_z(), // BHI
            0x9 => self.regs.get_c() || self.regs.get_z(), // BLS
            0xA => self.regs.get_n() == self.regs.get_v(), // BGE
            0xB => self.regs.get_n() != self.regs.get_v(), // BLT
            0xC => !self.regs.get_z() && self.regs.get_n() == self.regs.get_v(), // BGT
            0xD => self.regs.get_z() || self.regs.get_n() != self.regs.get_v(), // BLE
            _ => panic!("Invalid Opcode!"),
        };
        if condition {
            mmu.inc_clock(Cycle::N, self.regs.pc, 1);
            // Sub 4 since incremented 2 for next two instructions
            self.regs.pc = self.regs.pc.wrapping_sub(4).wrapping_add(offset);
            self.fill_thumb_instr_buffer(mmu);
        } else {
            mmu.inc_clock(Cycle::S, self.regs.pc, 1);
        }
    }

    // THUMB.17: software interrupt
    fn thumb_software_interrupt<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.17: software interrupt not implemented!")
    }

    // THUMB.18: unconditional branch
    fn uncond_branch<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.18: unconditional branch not implemented!")
    }

    // THUMB.19: long branch with link
    fn branch_with_link<M>(&mut self, instr: u16, mmu: &mut M) where M: IMMU {
        assert_eq!(instr >> 12, 0xF);
        let offset = (instr & 0x7FF) as u32;
        let offset = if offset >> 10 & 0x1 != 0 { 0xFFFF_F800 | offset } else { offset };
        if instr >> 11 & 0x1 != 0 { // Second Instruction
            mmu.inc_clock(Cycle::N, self.regs.pc, 1);
            let next_instr_pc = self.regs.pc.wrapping_sub(2);
            self.regs.pc = self.regs.get_reg(Reg::R14).wrapping_add(offset << 1);
            self.regs.set_reg(Reg::R14, next_instr_pc | 0x1);
            self.fill_thumb_instr_buffer(mmu);
        } else { // First Instruction
            assert_eq!(instr >> 11, 0b11110);
            self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_add(offset << 12));
            mmu.inc_clock(Cycle::S, self.regs.pc, 1);
        }
    }
}
