use super::*;
use crate::cpu::registers::{Reg, RegValues};
use crate::cpu::tests::*;

#[test]
fn test_immediate() {
    fn make_instr(opcode: u16, dest_reg: u16, immediate: u16) -> u16 {
        0b001 << 13 | opcode << 11 | dest_reg << 8 | immediate
    }

    // MOV
    let (cpu, mmu) = run_instr!(immediate, make_instr(0, 0, 0xFF), R0 = 0xF, CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0xFF, R15 = 2, CPSR = 0x20);
    assert_cycle_times(mmu, 1, 0, 0);

    // CMP
    let (cpu, mmu) = run_instr!(immediate, make_instr(1, 0, 0xFF), R0 = 0, CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0, R15 = 2, CPSR = 0x80000020);
    assert_cycle_times(mmu, 1, 0, 0);

    // ADD
    let (cpu, mmu) = run_instr!(immediate, make_instr(2, 0, 0xF), R0 = 0xFFFFFFFF, CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0xE, R15 = 2, CPSR = 0x20000020);
    assert_cycle_times(mmu, 1, 0, 0);

    // SUB
    let (cpu, mmu) = run_instr!(immediate, make_instr(3, 0, 0xF), R0 = 0xFFFFFFFF, CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0xFFFFFFF0, R15 = 2, CPSR = 0xA0000020);
    assert_cycle_times(mmu, 1, 0, 0);
}
