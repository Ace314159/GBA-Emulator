mod arm;
mod thumb;
mod registers;

use crate::mmu::IMMU;
use registers::RegValues;
use registers::Reg;

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
}
