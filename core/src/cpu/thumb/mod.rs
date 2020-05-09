use super::CPU;
use super::IMMU;
use super::registers::Reg;

use crate::mmu::Cycle;

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
            {:08X} {:08X} {:08X} {:08X} cpsr: {:08X} |     {:04X}",
            self.regs.get_reg(R0), self.regs.get_reg(R1), self.regs.get_reg(R2), self.regs.get_reg(R3),
            self.regs.get_reg(R4), self.regs.get_reg(R5), self.regs.get_reg(R6), self.regs.get_reg(R7),
            self.regs.get_reg(R8), self.regs.get_reg(R9), self.regs.get_reg(R10), self.regs.get_reg(R11),
            self.regs.get_reg(R12), self.regs.get_reg(R13), self.regs.get_reg(R14), self.regs.get_reg(R15),
            self.regs.get_reg(CPSR), instr);
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
    fn move_shifted_reg<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.1: move shifted register not implemented!")
    }

    // THUMB.2: add/subtract
    fn add_sub<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.2: add/subtract not implemented!")
    }

    // THUMB.3: move/compare/add/subtract immediate
    fn immediate<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.3: move/compare/add/subtract immediate not implemented!")
    }

    // THUMB.4: ALU operations
    fn alu<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.4: ALU operations not implemented!")
    }

    // THUMB.5: Hi register operations/branch exchange
    fn hi_reg_bx<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.5: Hi register operations/branch exchange not implemented!")
    }

    // THUMB.6: load PC-relative
    fn load_pc_rel<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.6: load PC-relative not implemented!")
    }

    // THUMB.7: load/store with register offset
    fn load_store_reg_offset<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.7: load/store with register offset not implemented!")
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
    fn load_store_halfword<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.10: load/store halfword not implemented!")
    }

    // THUMB.11: load/store SP-relative
    fn load_store_sp_rel<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.11: load/store SP-relative not implemented!")
    }

    // THUMB.12: get relative address
    fn get_rel_addr<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.12: get relative address not implemented!")
    }

    // THUMB.13: add offset to stack pointer
    fn add_offset_sp<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.13: add offset to stack pointer not implemented!")
    }

    // THUMB.14: push/pop registers
    fn push_pop_regs<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.14: push/pop registers not implemented!")
    }

    // THUMB.15: multiple load/store
    fn multiple_load_store<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.15: multiple load/store not implemented!")
    }

    // THUMB.16: conditional branch
    fn cond_branch<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.16: conditional branch not implemented!")
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
    fn branch_with_link<M>(&mut self, _instr: u16, _mmu: &mut M) where M: IMMU {
        unimplemented!("THUMB.19: long branch with link not implemented!")
    }
}
