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

    pub(super) fn emulate_thumb_instr<M>(&mut self, _mmu: &mut M) where M: IMMU {
        let instr = self.instr_buffer[0];
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
        panic!("STOP");
    }
}
