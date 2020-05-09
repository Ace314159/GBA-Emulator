use super::CPU;
use super::IMMU;
use super::registers::Reg;

use crate::mmu::Cycle;

impl CPU {
    pub(super) fn emulate_thumb_instr<M>(&mut self, _mmu: &mut M) where M: IMMU {
        unimplemented!("Thumb instruction set not implemented!")
    }

    pub(super) fn fill_thumb_instr_buffer<M>(&mut self, mmu: &mut M) where M: IMMU {
        self.instr_buffer[0] = mmu.read16(self.regs.pc & !0x1);
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
        self.regs.pc = self.regs.pc.wrapping_add(2);

        self.instr_buffer[1] = mmu.read16(self.regs.pc & !0x1);
        mmu.inc_clock(Cycle::S, self.regs.pc & !0x1, 1);
    }
}
