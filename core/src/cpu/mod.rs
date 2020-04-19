mod registers;

use crate::mmu::MMU;
use crate::mmu::Cycle;
use registers::RegValues;
use registers::Reg;


pub struct CPU {
    regs: RegValues,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            regs: RegValues::new(),
        }
    }

    pub fn emulate_instr(&mut self, mmu: &mut MMU) {
        if self.regs.get_t() { self.emulate_thumb_instr(mmu) }
        else { self.emulate_arm_instr(mmu) }
    }

    pub fn emulate_thumb_instr(&mut self, mmu: &mut MMU) {
        unimplemented!("Thumb instruction set not implemented!")
    }

    pub fn emulate_arm_instr(&mut self, mmu: &mut MMU) {
        let pc = self.regs.get_pc() & !0x3; // Align pc
        self.regs.set_pc(pc.wrapping_add(4));
        let instr = mmu.read32(pc);

        if self.should_exec(instr) {
            if (instr >> 4) & 0xFFFFFF == 0b0001_0010_1111_1111_1111 { self.branch_and_exchange(instr) }
            else if (instr >> 25) & 0x7 == 0b101 { self.branch_branch_with_link(instr) }
            else if (instr >> 26) & 0x3 == 0b00 { self.data_proc_psr_transfer(instr) }
            else if (instr >> 22) & 0x3F == 0b00_0000 { self.mul(instr) }
            else if (instr >> 23) & 0x1F == 0b0_0001 { self.mul_long(instr) }
            else if (instr >> 26) & 0x3 == 0b01 { self.single_data_transfer(instr) }
            else if (instr >> 25) & 0x7 == 0b000 { self.halfword_and_signed_data_transfer(instr) }
            else if (instr >> 25) & 0x7 == 0b100 { self.block_data_transfer(instr) }
            else if (instr >> 23) & 0x1F == 0b0_0010 { self.single_data_swap(instr) }
            else if (instr >> 24) & 0xF == 0b1111 { self.software_interrupt(instr) }
            else if (instr >> 24) & 0xF == 0b1110 { self.coprocessor(instr) }
            else if (instr >> 25) & 0x7 == 0b110 { self.coprocessor(instr) }
            else if (instr >> 25) & 0x7 == 0b011 { self.undefined_instr(instr) }
            else { panic!("Unexpected instruction") }
        } else {
            mmu.inc_clock(1, Cycle::S, pc);
        }
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
    fn branch_and_exchange(&mut self, instr: u32) {
        unimplemented!("ARM.3: Branch and Exchange (BX) not implemented!");
    }

    // ARM.4: Branch and Branch with Link (B, BL)
    fn branch_branch_with_link(&mut self, instr: u32) {
        let opcode = (instr >> 24) & 0x1;
        let offset = instr & 0xFF_FFFF;
        let offset = if (offset >> 23) == 1 { 0xFF00_0000 | offset } else { offset };
        let pc = self.regs.get_pc();

        if opcode == 1 { self.regs.set_reg(Reg::R14, pc) } // Branch with Link
        self.regs.set_pc(pc.wrapping_add(offset * 4));
    }

    // ARM.5: Data Processing
    // ARM.6: PSR Transfer (MRS, MSR)
    fn data_proc_psr_transfer(&mut self, instr: u32) {
        unimplemented!("ARM.5: Data Processing and ARM.6: PSR Transfer (MRS, MSR) not implemented!");
    }

    // ARM.7: Multiply and Multiply-Accumulate (MUL, MLA)
    fn mul(&mut self, instr: u32) {
        unimplemented!("ARM.7: Multiply and Multiply-Accumulate (MUL, MLA) not implemented!");
    }

    // ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL)
    fn mul_long(&mut self, instr: u32) {
        unimplemented!("ARM.8: Multiply Long and Multiply-Accumulate Long (MULL, MLAL) not implemented!");
    }

    // ARM.9: Single Data Transfer (LDR, STR)
    fn single_data_transfer(&mut self, instr: u32) {
        unimplemented!("ARM.9: Single Data Transfer (LDR, STR) not implemented!");
    }

    // ARM.10: Halfword and Signed Data Transfer (STRH,LDRH,LDRSB,LDRSH)
    fn halfword_and_signed_data_transfer(&mut self, instr: u32) {
        unimplemented!("ARM.10: Halfword and Signed Data Transfer (STRH,LDRH,LDRSB,LDRSH) not implemented!");
    }

    // ARM.11: Block Data Transfer (LDM,STM)
    fn block_data_transfer(&mut self, instr: u32) {
        unimplemented!("ARM.11: Block Data Transfer (LDM,STM) not implemented!");
    }

    // ARM.12: Single Data Swap (SWP)
    fn single_data_swap(&mut self, instr: u32) {
        unimplemented!("ARM.12: Single Data Swap (SWP) not implemented!");
    }

    // ARM.13: Software Interrupt (SWI)
    fn software_interrupt(&mut self, instr: u32) {
        unimplemented!("ARM.13: Software Interrupt (SWI) not implemented!");
    }

    // ARM.14: Coprocessor Data Operations (CDP)
    // ARM.15: Coprocessor Data Transfers (LDC,STC)
    // ARM.16: Coprocessor Register Transfers (MRC, MCR)
    fn coprocessor(&mut self, instr: u32) {
        unimplemented!("Coprocessor not implemented!");
    }

    // ARM.17: Undefined Instruction
    fn undefined_instr(&mut self, instr: u32) {
        unimplemented!("ARM.17: Undefined Instruction not implemented!");
    }
}
