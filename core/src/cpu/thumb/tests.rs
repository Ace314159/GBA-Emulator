use super::*;
use crate::cpu::registers::{Reg, RegValues};
use crate::cpu::tests::*;
use std::collections::HashMap;

#[test]
// THUMB.3: move/compare/add/subtract immediate
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

#[test]
// THUMB.6: load PC-relative
fn test_load_pc_rel() {
    fn make_instr(dest_reg: u16, offset: u16) -> u16 {
        0b01001 << 11 | dest_reg << 8 | offset
    }

    // LDR r0, [pc, #0xFF]
    let (cpu, mmu) = run_instr!(load_pc_rel, make_instr(0, 0xFF), CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0x400, R15 = 2, CPSR = 0x20);
    assert_cycle_times(mmu, 1, 1, 1);
}

#[test]
// THUMB.14: push/pop registers
fn test_push_pop_regs() {
    fn make_instr(pop: bool, pc_lr: bool, r_list: u16) -> u16 {
        0b1011 << 12 | (pop as u16) << 11 | 0b10 << 9 | (pc_lr as u16) << 8 | r_list
    }

    // POP {R3}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(true, false, 1 << 3), CPSR = 0x20);
    assert_regs!(cpu.regs, R3 = 0x3007F00, R13 = 0x3007F00 + 4, R15 = 2, CPSR = 0x20);
    assert_cycle_times(mmu, 1, 1, 1);

    // POP {PC}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(true, true, 0), CPSR = 0x20);
    assert_regs!(cpu.regs, R15 = 0x3007F00, R13 = 0x3007F00 + 4, CPSR = 0x20);
    assert_cycle_times(mmu, 2, 1, 2);

    // POP {R0,R5,R7}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(true, false, 1 << 7 | 1 << 5 | 1), CPSR = 0x20);
    assert_regs!(cpu.regs, R0 = 0x3007F00, R5 = 0x3007F00 + 4, R7 = 0x3007F00 + 8, R13 = 0x3007F00 + 12, R15 = 2, CPSR = 0x20);
    assert_cycle_times(mmu, 3, 1, 1);

    // POP {R1,R2,R5,R6,PC}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(true, true,
        1 << 6 | 1 << 5 | 1 << 2 | 1 << 1), CPSR = 0x20);
    assert_regs!(cpu.regs, R1 = 0x3007F00, R2 = 0x3007F00 + 4, R5 = 0x3007F00 + 8, R6 = 0x3007F00 + 12,
        R13 = 0x3007F00 + 20, R15 = 0x3007F00 + 16, CPSR = 0x20);
    assert_cycle_times(mmu, 6, 1, 2);

    // PUSH {R0-R7,LR}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(false, false, 0xFF), 
    R0 = 1, R1 = 2, R2 = 3, R3 = 4, R4 = 5, R5 = 6, R6 = 7, R7 = 8, CPSR = 0x20);
    assert_regs!(cpu.regs, R15 = 2, R13 = 0x3007F00 - 4 * 8,
        R0 = 1, R1 = 2, R2 = 3, R3 = 4, R4 = 5, R5 = 6, R6 = 7, R7 = 8, CPSR = 0x20);
    assert_writes!(mmu.writes32, 0x3007F00 - 4 => 8, 0x3007F00 - 8 => 7, 0x3007F00 - 12 => 6,
        0x3007F00 - 16 => 5, 0x3007F00 - 20 => 4, 0x3007F00 - 24 => 3, 0x3007F00 - 28 => 2, 0x3007F00 - 32 => 1);
    assert_cycle_times(mmu, 7, 0, 2);

    // PUSH {LR}
    let (cpu, mmu) = run_instr!(push_pop_regs, make_instr(false, true, 0), R14 = 100);
    assert_writes!(mmu.writes32, 0x3007F00 - 4 => 100);
    assert_cycle_times(mmu, 0, 0, 2);
}
