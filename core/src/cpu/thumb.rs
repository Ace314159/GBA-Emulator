use super::CPU;
use super::IMMU;

impl CPU {
    pub(super) fn emulate_thumb_instr<M>(&mut self, _mmu: &mut M) where M: IMMU {
        unimplemented!("Thumb instruction set not implemented!")
    }
}
