use super::CPU;
use super::IIO;
use super::registers::{Mode, Reg};

use crate::io::Cycle;

#[cfg(test)]
mod tests;

impl CPU {
    pub(super) fn fill_thumb_instr_buffer<I>(&mut self, io: &mut I) where I: IIO {
        self.regs.pc &= !0x1;
        self.instr_buffer[0] = io.read16(self.regs.pc & !0x1) as u32;
        io.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
        self.regs.pc = self.regs.pc.wrapping_add(2);

        self.instr_buffer[1] = io.read16(self.regs.pc & !0x1) as u32;
        io.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
    }

    pub(super) fn emulate_thumb_instr<I>(&mut self, io: &mut I) where I: IIO {
        let instr = self.instr_buffer[0] as u16;
        {
            use Reg::*;
            trace!("{:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} \
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
        self.instr_buffer[1] = io.read16(self.regs.pc & !0x1) as u32;

        if instr & 0b1111_1000_0000_0000 == 0b0001_1000_0000_0000 { self.add_sub(io, instr) }
        else if instr & 0b1110_0000_0000_0000 == 0b0000_0000_0000_0000 { self.move_shifted_reg(io, instr) }
        else if instr & 0b1110_0000_0000_0000 == 0b0010_0000_0000_0000 { self.immediate(io, instr) }
        else if instr & 0b1111_1100_0000_0000 == 0b0100_0000_0000_0000 { self.alu(io, instr) }
        else if instr & 0b1111_1100_0000_0000 == 0b0100_0100_0000_0000 { self.hi_reg_bx(io, instr) }
        else if instr & 0b1111_1000_0000_0000 == 0b0100_1000_0000_0000 { self.load_pc_rel(io, instr) }
        else if instr & 0b1111_0010_0000_0000 == 0b0101_0000_0000_0000 { self.load_store_reg_offset(io, instr) }
        else if instr & 0b1111_0010_0000_0000 == 0b0101_0010_0000_0000 { self.load_store_sign_ext(io, instr) }
        else if instr & 0b1110_0000_0000_0000 == 0b0110_0000_0000_0000 { self.load_store_imm_offset(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1000_0000_0000_0000 { self.load_store_halfword(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1001_0000_0000_0000 { self.load_store_sp_rel(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1010_0000_0000_0000 { self.get_rel_addr(io, instr) }
        else if instr & 0b1111_1111_0000_0000 == 0b1011_0000_0000_0000 { self.add_offset_sp(io, instr) }
        else if instr & 0b1111_0110_0000_0000 == 0b1011_0100_0000_0000 { self.push_pop_regs(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1100_0000_0000_0000 { self.multiple_load_store(io, instr) }
        else if instr & 0b1111_1111_0000_0000 == 0b1101_1111_0000_0000 { self.thumb_software_interrupt(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1101_0000_0000_0000 { self.cond_branch(io, instr) }
        else if instr & 0b1111_1000_0000_0000 == 0b1110_0000_0000_0000 { self.uncond_branch(io, instr) }
        else if instr & 0b1111_0000_0000_0000 == 0b1111_0000_0000_0000 { self.branch_with_link(io, instr) }
        else { panic!("Unexpected Instruction: {:08X}", instr) }
    }
    
    // THUMB.1: move shifted register
    fn move_shifted_reg<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 13, 0b000);
        let opcode = (instr >> 11 & 0x3) as u32;
        let offset = (instr >> 6 & 0x1F) as u32;
        let src = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let dest_reg = (instr & 0x7) as u32;
        assert_ne!(opcode, 0b11);
        let result = self.shift(io, opcode, src, offset, true, true);

        self.regs.set_n(result & 0x8000_0000 != 0);
        self.regs.set_z(result == 0);
        self.regs.set_reg_i(dest_reg, result);
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.2: add/subtract
    fn add_sub<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
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
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.3: move/compare/add/subtract immediate
    fn immediate<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
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
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.4: ALU operations
    fn alu<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 10 & 0x3F, 0b010000);
        let opcode = instr >> 6 & 0xF;
        let src = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let dest_reg = (instr & 0x7) as u32;
        let dest = self.regs.get_reg_i(dest_reg);
        let result = match opcode {
            0x0 => dest & src, // AND
            0x1 => dest ^ src, // XOR 
            0x2 => self.shift(io, 0, dest, src & 0xFF, false, true), // LSL
            0x3 => self.shift(io, 1, dest, src & 0xFF, false, true), // LSR
            0x4 => self.shift(io, 2, dest, src & 0xFF, false, true), // ASR
            0x5 => self.adc(dest, src, true), // ADC
            0x6 => self.sbc(dest, src, true), // SBC
            0x7 => self.shift(io, 3, dest, src & 0xFF, false, true), // ROR
            0x8 => dest & src, // TST
            0x9 => self.sub(0, src, true), // NEG
            0xA => self.sub(dest, src, true), // CMP
            0xB => self.add(dest, src, true), // CMN
            0xC => dest | src, // ORR
            0xD => { self.inc_mul_clocks(io, dest, true); dest.wrapping_mul(src) }, // MUL
            0xE => dest & !src, // BIC
            0xF => !src, // MVN
            _ => panic!("Invalid opcode!"),
        };
        self.regs.set_n(result & 0x8000_0000 != 0);
        self.regs.set_z(result == 0);

        if ![0x8, 0xA, 0xB].contains(&opcode) { self.regs.set_reg_i(dest_reg, result) }
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.5: Hi register operations/branch exchange
    fn hi_reg_bx<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
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
                io.inc_clock(Cycle::N, self.regs.pc, 1);
                if src & 0x1 != 0 {
                    self.regs.pc = self.regs.pc & !0x1;
                    self.fill_thumb_instr_buffer(io);
                } else {
                    self.regs.pc = self.regs.pc & !0x2;
                    self.regs.set_t(false);
                    self.fill_arm_instr_buffer(io);
                }
                return
            },
            _ => panic!("Invalid Opcode!"),
        };
        if opcode & 0x1 == 0 { self.regs.set_reg_i(dest_reg, result) }
        if dest_reg == 15 {
            io.inc_clock(Cycle::N, self.regs.pc, 1);
            self.fill_thumb_instr_buffer(io);
        } else { io.inc_clock(Cycle::S, self.regs.pc, 1) }
    }

    // THUMB.6: load PC-relative
    fn load_pc_rel<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 11, 0b01001);
        let dest_reg = (instr >> 8 & 0x7) as u32;
        let offset = (instr & 0xFF) as u32;
        let addr = (self.regs.pc & !0x2).wrapping_add(offset * 4);
        self.regs.set_reg_i(dest_reg, io.read32(addr & !0x3).rotate_right((addr & 0x3) * 8) as u32);
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        io.inc_clock(Cycle::I, 0, 0);
        io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
    }

    // THUMB.7: load/store with register offset
    fn load_store_reg_offset<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0b0101);
        let opcode = instr >> 10 & 0x3; 
        assert_eq!(instr >> 9 & 0x1, 0);
        let offset_reg = (instr >> 6 & 0x7) as u32;
        let base_reg = (instr >> 3 & 0x7) as u32;
        let addr = self.regs.get_reg_i(base_reg).wrapping_add(self.regs.get_reg_i(offset_reg));
        let src_dest_reg = (instr & 0x7) as u32;
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if opcode & 0b10 != 0 { // Load
            io.inc_clock(Cycle::I, 0, 0);
            io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, if opcode & 0b01 != 0 {
                io.read8(addr) as u32 // LDRB
            } else {
                io.read32(addr & !0x3).rotate_right((addr & 0x3) * 8) // LDR
            });

        } else { // Store
            let access_width = if opcode & 0b01 != 0 { // STRB
                io.write8(addr, self.regs.get_reg_i(src_dest_reg) as u8);
                1
            } else { // STR
                io.write32(addr & !0x3, self.regs.get_reg_i(src_dest_reg));
                0
            };
            io.inc_clock(Cycle::N, addr, access_width);
        }
    }

    // THUMB.8: load/store sign-extended byte/halfword
    fn load_store_sign_ext<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0b0101);
        let opcode = instr >> 10 & 0x3;
        assert_eq!(instr >> 9 & 0x1, 1);
        let offset_reg = (instr >> 6 & 0x7) as u32;
        let base_reg = (instr >> 3 & 0x7) as u32;
        let src_dest_reg = (instr & 0x7) as u32;
        let addr = self.regs.get_reg_i(base_reg).wrapping_add(self.regs.get_reg_i(offset_reg));

        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if opcode == 0 { // STRH
            io.inc_clock(Cycle::N, addr, 1);
            io.write16(addr & !0x1, self.regs.get_reg_i(src_dest_reg) as u16);
        } else { // Load
            io.inc_clock(Cycle::I, 0, 0);
            io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, match opcode {
                1 => io.read8(addr) as i8 as u32,
                2 => (io.read16(addr & !0x1) as u32).rotate_right((addr & 0x1) * 8),
                3 if addr & 0x1 == 1 => io.read8(addr) as i8 as u32,
                3 => io.read16(addr) as i16 as u32,
                _ => panic!("Invalid opcode!")
            });
        }
    }

    // THUMB.9: load/store with immediate offset
    fn load_store_imm_offset<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 13, 0b011);
        let load = instr >> 11 & 0x1 != 0;
        let byte = instr >> 12 & 0x1 != 0;
        let offset = (instr >> 6 & 0x1F) as u32;
        let base = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let src_dest_reg = (instr & 0x7) as u32;

        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if load {
            io.inc_clock(Cycle::I, 0, 0);
            io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, if byte {
                let addr = base.wrapping_add(offset);
                io.read8(addr) as u32
            } else {
                let addr = base.wrapping_add(offset << 2);
                io.read32(addr & !0x3).rotate_right((addr & 0x3) * 8)
            });
        } else {
            let value = self.regs.get_reg_i(src_dest_reg);
            let addr = if byte {
                let addr = base.wrapping_add(offset);
                io.write8(addr, value as u8);
                addr
            } else {
                let addr = base.wrapping_add(offset << 2);
                io.write32(addr & !0x3, value);
                addr
            };
            io.inc_clock(Cycle::N, addr, 1);
        }
    }

    // THUMB.10: load/store halfword
    fn load_store_halfword<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0b1000);
        let load = instr >> 11 & 0x1 != 0;
        let offset = (instr >> 6 & 0x1F) as u32;
        let base = self.regs.get_reg_i((instr >> 3 & 0x7) as u32);
        let src_dest_reg = (instr & 0x7) as u32;
        let addr = base + offset * 2;

        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if load {
            io.inc_clock(Cycle::I, 0, 0);
            io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
            self.regs.set_reg_i(src_dest_reg, (io.read16(addr & !0x1) as u32).rotate_right((addr & 0x1) * 8));
        } else {
            io.inc_clock(Cycle::N, addr & 0x1, 1);
            io.write16(addr & !0x1, self.regs.get_reg_i(src_dest_reg) as u16);
        }
    }

    // THUMB.11: load/store SP-relative
    fn load_store_sp_rel<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12 & 0xF, 0b1001);
        let load = instr >> 11 & 0x1 != 0;
        let src_dest_reg = (instr >> 8 & 0x7) as u32;
        let offset = (instr & 0xFF) * 4;
        let addr = self.regs.get_reg(Reg::R13).wrapping_add(offset as u32);
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if load {
            io.inc_clock(Cycle::I, 0, 0);
            self.regs.set_reg_i(src_dest_reg, io.read32(addr & !0x3).rotate_right((addr & 0x3) * 8));
            io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1);
        } else {
            io.inc_clock(Cycle::N, addr, 2);
            io.write32(addr & !0x3, self.regs.get_reg_i(src_dest_reg));
        }
    }

    // THUMB.12: get relative address
    fn get_rel_addr<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12 & 0xF, 0b1010);
        let src = if instr >> 11 & 0x1 != 0 { // SP
            self.regs.get_reg(Reg::R13)
        } else { // PC
            self.regs.pc & !0x2
        };
        let dest_reg = (instr >> 8 & 0x7) as u32;
        let offset = (instr & 0xFF) as u32;
        self.regs.set_reg_i(dest_reg, src.wrapping_add(offset * 4));
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.13: add offset to stack pointer
    fn add_offset_sp<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 8 & 0xFF, 0b10110000);
        let sub = instr >> 7 & 0x1 != 0;
        let offset = ((instr & 0x7F) * 4) as u32;
        let sp = self.regs.get_reg(Reg::R13);
        let value = if sub { sp.wrapping_sub(offset) } else { sp.wrapping_add(offset) };
        self.regs.set_reg(Reg::R13, value);
        io.inc_clock(Cycle::S, self.regs.pc, 1);
    }

    // THUMB.14: push/pop registers
    fn push_pop_regs<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12 & 0xF, 0b1011);
        let pop = instr >> 11 & 0x1 != 0;
        assert_eq!(instr >> 9 & 0x3, 0b10);
        let pc_lr = instr >> 8 & 0x1 != 0;
        let mut r_list = (instr & 0xFF) as u8;
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        if pop {
            let mut stack_pop = |sp, last_access| {
                let value = io.read32(sp);
                if last_access { io.inc_clock(Cycle::I, 0, 0) }
                else { io.inc_clock(Cycle::S, sp, 2) }
                value
            };
            let mut reg = 0;
            let mut sp = self.regs.get_reg(Reg::R13);
            while r_list != 0 {
                if r_list & 0x1 != 0 {
                    let value = stack_pop(sp, r_list == 1 && !pc_lr);
                    self.regs.set_reg_i(reg, value);
                    sp += 4;
                }
                reg += 1;
                r_list >>= 1;
            }
            if pc_lr {
                self.regs.pc = stack_pop(sp, true) & !0x1;
                sp += 4;
                io.inc_clock(Cycle::N, self.regs.pc.wrapping_add(2), 1);
                self.fill_thumb_instr_buffer(io);
            } else { io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1); }
            self.regs.set_reg(Reg::R13, sp);
        } else {
            let mut stack_push = |sp, value, last_access| {
                io.write32(sp, value);
                if last_access { io.inc_clock(Cycle::N, sp, 2) }
                else { io.inc_clock(Cycle::S, sp, 2) }
            };
            let mut reg = 0;
            let initial_sp = self.regs.get_reg(Reg::R13);
            let mut sp = self.regs.get_reg(Reg::R13).wrapping_sub(4 * (r_list.count_ones() + pc_lr as u32));
            self.regs.set_reg(Reg::R13, sp);
            while r_list != 0 {
                if r_list & 0x1 != 0 {
                    stack_push(sp, self.regs.get_reg_i(reg), r_list == 0x1 && !pc_lr);
                    sp += 4;
                }
                reg += 1;
                r_list >>= 1;
            }
            if pc_lr { stack_push(sp, self.regs.get_reg(Reg::R14), true); sp += 4}
            assert_eq!(initial_sp, sp);
        }
    }

    // THUMB.15: multiple load/store
    fn multiple_load_store<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0b1100);
        let load = instr >> 11 & 0x1 != 0;
        let base_reg = (instr >> 8 & 0x7) as u32;
        let mut base = self.regs.get_reg_i(base_reg);
        let base_offset = base & 0x3;
        base -= base_offset;
        let mut r_list = (instr & 0xFF) as u8;
    
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        let mut reg = 0;
        let mut first = true;
        let final_base = base.wrapping_add(4 * r_list.count_ones()) + base_offset;
        if !load { self.regs.pc = self.regs.pc.wrapping_add(2); }
        let mut exec = |reg, last_access| {
            let addr = base;
            base = base.wrapping_add(4);
            if load {
                self.regs.set_reg_i(reg, io.read32(addr));
                if last_access { io.inc_clock(Cycle::I, 0, 0) }
                else { io.inc_clock(Cycle::S, addr, 2) }
            } else {
                io.write32(addr, self.regs.get_reg_i(reg));
                if last_access { io.inc_clock(Cycle::N, addr, 2) }
                else { io.inc_clock(Cycle::S, addr, 2) }
                if first { self.regs.set_reg_i(base_reg, final_base); first = false }
            }
        };
        if r_list == 0 {
            exec(15, true);
            if load {
                self.fill_thumb_instr_buffer(io);
            }
            base = base.wrapping_add(0x3C + base_offset);
        } else {
            while r_list != 0x1 {
                if r_list & 0x1 != 0 {
                    exec(reg, false);
                }
                reg += 1;
                r_list >>= 1;
            }
            exec(reg, true);
        }
        if load { io.inc_clock(Cycle::S, self.regs.pc.wrapping_add(2), 1) }
        else { self.regs.pc = self.regs.pc.wrapping_sub(2) }
        self.regs.set_reg_i(base_reg, base + base_offset);
    }

    // THUMB.16: conditional branch
    fn cond_branch<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0b1101);
        let condition = instr >> 8 & 0xF;
        assert_eq!(condition < 0xE, true);
        let offset = (instr & 0xFF) as i8 as u32;
        if self.should_exec(condition as u32) {
            io.inc_clock(Cycle::N, self.regs.pc, 1);
            self.regs.pc = self.regs.pc.wrapping_add(offset.wrapping_mul(2));
            self.fill_thumb_instr_buffer(io);
        } else {
            io.inc_clock(Cycle::S, self.regs.pc, 1);
        }
    }

    // THUMB.17: software interrupt
    fn thumb_software_interrupt<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 8 & 0xFF, 0b11011111);
        io.inc_clock(Cycle::N, self.regs.pc, 1);
        self.regs.change_mode(Mode::SVC);
        self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_sub(2));
        self.regs.set_t(false);
        self.regs.set_i(true);
        self.regs.pc = 0x8;
        self.fill_arm_instr_buffer(io);
    }

    // THUMB.18: unconditional branch
    fn uncond_branch<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 11, 0b11100);
        let offset = (instr & 0x7FF) as u32;
        let offset = if offset >> 10 & 0x1 != 0 { 0xFFFF_F800 | offset } else { offset };

        io.inc_clock(Cycle::N, self.regs.pc, 1);
        self.regs.pc = self.regs.pc.wrapping_add(offset << 1);
        self.fill_thumb_instr_buffer(io);
    }

    // THUMB.19: long branch with link
    fn branch_with_link<I>(&mut self, io: &mut I, instr: u16) where I: IIO {
        assert_eq!(instr >> 12, 0xF);
        let offset = (instr & 0x7FF) as u32;
        if instr >> 11 & 0x1 != 0 { // Second Instruction
            io.inc_clock(Cycle::N, self.regs.pc, 1);
            let next_instr_pc = self.regs.pc.wrapping_sub(2);
            self.regs.pc = self.regs.get_reg(Reg::R14).wrapping_add(offset << 1);
            self.regs.set_reg(Reg::R14, next_instr_pc | 0x1);
            self.fill_thumb_instr_buffer(io);
        } else { // First Instruction
            let offset = if offset >> 10 & 0x1 != 0 { 0xFFFF_F800 | offset } else { offset };
            assert_eq!(instr >> 11, 0b11110);
            self.regs.set_reg(Reg::R14, self.regs.pc.wrapping_add(offset << 12));
            io.inc_clock(Cycle::S, self.regs.pc, 1);
        }
    }
}
