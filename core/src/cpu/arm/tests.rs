use super::*;
use crate::cpu::tests::*;
use crate::cpu::registers::RegValues;
use std::collections::HashMap;

#[test]
fn test_arm_shift() {
    fn run_shift(shift_type: u32, operand: u32, shift: u32, immediate: bool, change_status: bool) -> (CPU, u32) {
        let mut mmu = TestMMU::new();
        let mut cpu = CPU::new(&mut mmu);
        let val = cpu.arm_shift(shift_type, operand, shift, immediate, change_status);
        (cpu, val)
    }
    // LSL #0
    let (cpu, val) = run_shift(0, 0xA, 0, true, true);
    assert_regs!(cpu.regs, R15 = 0);
    assert_eq!(val, 0xA);

    // LSR #0
    let (cpu, val) = run_shift(1, 0xFFFFFFFF, 0, true, true);
    assert_regs!(cpu.regs, R15 = 0, CPSR = 0x20000000);
    assert_eq!(val, 0);

    // ASR #0
    let (cpu, val) = run_shift(2, 0xFFFFFFFF, 0, true, true);
    assert_regs!(cpu.regs, R15 = 0, CPSR = 0x20000000);
    assert_eq!(val, 0xFFFFFFFF);

    // ROR #0
    let (cpu, val) = run_shift(3, 0xFFFFFFFF, 0, true, true);
    assert_regs!(cpu.regs, R15 = 0, CPSR = 0x20000000);
    assert_eq!(val, 0x7FFFFFFF);
}

#[test]
// ARM.3: Branch and Exchange (BX)
fn test_branch_and_exchange() {
    fn make_instr(operand_reg: u32) -> u32{
        0b1110 << 28 | 0b0001_0010_1111_1111_1111_0001 << 4 | operand_reg
    }

    // Switch to thumb
    let (cpu, mmu) = run_instr!(branch_and_exchange, make_instr(0), R0 = 0xFD);
    assert_regs!(cpu.regs, R0 = 0xFD, R15 = 0xFC, CPSR = 0x20);
    assert_cycle_times(mmu, 2, 0, 1);

    // Stay in arm
    let (cpu, mmu) = run_instr!(branch_and_exchange, make_instr(0), R0 = 0xFC);
    assert_regs!(cpu.regs, R0 = 0xFC, R15 = 0xFC);
    assert_cycle_times(mmu, 2, 0, 1);
}

#[test]
// ARM.4: Branch and Branch with Link (B, BL)
fn test_branch_branch_with_link() {
    fn make_instr(with_link: bool, offset: u32) -> u32 {
        0b1110 << 28 | 0b101 << 25 | (with_link as u32) << 24 | offset
    }

    // Offset 0
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(false, 0),);
    assert_regs!(cpu.regs, R15 = 0x8);
    assert_cycle_times(mmu, 2, 0, 1);

    // Link Functionality
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(true, 0),);
    assert_regs!(cpu.regs, R14 = 4, R15 = 0x8);
    assert_cycle_times(mmu, 2, 0, 1);

    // Offset 1
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(false, 1),);
    assert_regs!(cpu.regs, R15 = 0xC);
    assert_cycle_times(mmu, 2, 0, 1);

    // Offset -1
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(false, 0xFFFFFF),);
    assert_regs!(cpu.regs, R15 = 0x4);
    assert_cycle_times(mmu, 2, 0, 1);

    // Offset 0x7FFFFF - max offset
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(false, 0x7FFFFF),);
    assert_regs!(cpu.regs, R15 = 0x7FFFFF * 4 + 8);
    assert_cycle_times(mmu, 2, 0, 1);

    // Offset 0x800000 - min offset
    let (cpu, mmu) = run_instr!(branch_branch_with_link, make_instr(false, 0x800000),);
    assert_regs!(cpu.regs, R15 = 0xFE0_00000u32 + 8);
    assert_cycle_times(mmu, 2, 0, 1);
}

#[test]
// ARM.5: Data Processing
fn test_data_proc() {
    fn make_immediate(opcode: u32, set_status: bool, op1_reg: u32, dest: u32, shift: u32, op2: u32) -> u32 {
        0b1110 << 28 | 0b00 << 26 | (true as u32) << 25 | opcode << 21 | (set_status as u32) << 20 |
        op1_reg << 16 | dest << 12 | shift << 8 | op2 
    }

    // AND
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0, false, 0, 0, 0, 1),
    R0 = 0xFFF);
    assert_regs!(cpu.regs, R0 = 1, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // EOR
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(1, false, 0, 0, 0, 0xAC),
    R0 = 0xFF);
    assert_regs!(cpu.regs, R0 = 0x53, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // SUB
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(2, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 400, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // RSB
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(3, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = !400 + 1, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // ADD
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(4, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 600, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // ADC
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(5, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 600, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(5, false, 0, 0, 0, 100),
    R0 = 500, CPSR = 0x20000000);
    assert_regs!(cpu.regs, R0 = 601, R15 = 4, CPSR = 0x20000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // SBC
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(6, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 399, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(6, false, 0, 0, 0, 100),
    R0 = 500, CPSR = 0x20000000);
    assert_regs!(cpu.regs, R0 = 400, R15 = 4, CPSR = 0x20000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // RSC
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(7, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = !401 + 1, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(7, false, 0, 0, 0, 100),
    R0 = 500, CPSR = 0x20000000);
    assert_regs!(cpu.regs, R0 = !400 + 1, R15 = 4, CPSR = 0x20000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // TST
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(8, true, 0, 0, 0, 1),
    R0 = 0xFFF);
    assert_regs!(cpu.regs, R0 = 0xFFF, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(8, true, 0, 0, 0, 0),
    R0 = 0xFFF);
    assert_regs!(cpu.regs, R0 = 0xFFF, R15 = 4, CPSR = 0x40000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // TEQ
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(9, true, 0, 0, 0, 0xAB),
    R0 = 0xFF);
    assert_regs!(cpu.regs, R0 = 0xFF, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(9, true, 0, 0, 0, 0xAB),
    R0 = 0xAB);
    assert_regs!(cpu.regs, R0 = 0xAB, R15 = 4, CPSR = 0x40000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // CMP
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xA, true, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 500, R15 = 4, CPSR = 0x20000000);
    assert_cycle_times(mmu, 1, 0, 0);
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xA, true, 0, 0, 0, 100),
    R0 = 100);
    assert_regs!(cpu.regs, R0 = 100, R15 = 4, CPSR = 0x60000000);
    assert_cycle_times(mmu, 1, 0, 0);
    // CMN
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xB, true, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 500, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // ORR
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xC, false, 0, 0, 0, 100),);
    assert_regs!(cpu.regs, R0 = 100, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // MOV
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xD, true, 0, 0, 0, 100),);
    assert_regs!(cpu.regs, R0 = 100, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // BIC
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xE, false, 0, 0, 0, 100),
    R0 = 500);
    assert_regs!(cpu.regs, R0 = 400, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
    // MVN
    let (cpu, mmu) = run_instr!(data_proc, make_immediate(0xF, false, 0, 0, 0, 100),);
    assert_regs!(cpu.regs, R0 = !100, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);

    println!("Second Set");
    fn make_reg_instr(opcode: u32, set_status: bool, op1_reg: u32, dest: u32, shift: u32,
        shift_by_type: u32, shift_by_reg: bool, op2: u32) -> u32 {
        0b1110 << 28 | 0b00 << 26 | (false as u32) << 25 | opcode << 21 | (set_status as u32) << 20 |
        op1_reg << 16 | dest << 12 | if shift_by_reg { shift << 8 | 0 << 7 } else { shift << 7 } |
        shift_by_type << 5 | (shift_by_reg as u32) << 4 | op2
    }
    // MOV r0, r0 LSL r0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 0, true, 0),);
    assert_regs!(cpu.regs, R15 = 4, CPSR = 0x40000000);
    assert_cycle_times(mmu, 1, 1, 0);

    // MOV pc, r0, LSL r0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, false, 0, 15, 0, 0, true, 0),
    R0 = 2);
    assert_regs!(cpu.regs, R0 = 2, R15 = 8);
    assert_cycle_times(mmu, 2, 1, 1);

    // MOV r0, pc, LSL r0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, false, 0, 0, 0, 0, true, 15),
    R0 = 0);
    assert_regs!(cpu.regs, R0 = 12, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 0);

    // MOV r0, pc
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, false, 0, 0, 0, 0, false, 15),);
    assert_regs!(cpu.regs, R0 = 8, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
}

#[test]
// ARM.6: PSR Transfer (MRS, MSR)
fn test_psr_transfer() {
    fn make_mrs(spsr: bool, dest_reg: u32) -> u32 {
        0b1110 << 28 | 0b00 << 26 | (false as u32) << 25 | 0b10 << 23 | (spsr as u32) << 22 |
        (false as u32) << 21 | 0xF << 16 | dest_reg << 12
    }

    // MRS r0, cpsr
    let (cpu, mmu) = run_instr!(psr_transfer, make_mrs(false, 0),);
    assert_regs!(cpu.regs, R0 = 0x1F, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);

    fn make_msr(immediate_operand: bool, spsr: bool, f: bool, s: bool, x: bool, c: bool, operand: u32) -> u32 {
        0b1110 << 28 | 0b00 << 26 | (immediate_operand as u32) << 25 | 0b10 << 23 | (spsr as u32) << 22 |
        (true as u32) << 21 | (f as u32) << 19 | (s as u32) << 18 | (x as u32) << 17 | (c as u32) << 16 |
        0xF << 12 | operand
    }

    // MSR cpsr, r0
    let (cpu, mmu) = run_instr!(psr_transfer, make_msr(false, false, true, false, false, false, 0),
    R0 = 0xFFFFFFFF);
    assert_regs!(cpu.regs, R0 = 0xFFFFFFFF, R15 = 4, CPSR = 0xFF00001F);
    assert_cycle_times(mmu, 1, 0, 0);
}

#[test]
// ARM.9: Single Data Transfer (LDR, STR)
fn test_single_data_transfer() {
    fn make_instr(pre_offset: bool, add_offset: bool, transfer_byte: bool, load: bool, write_back: bool,
        base_reg: u32, src_dest_reg: u32, offset: u32) -> u32 {
        0b1110 << 28 | 0b01 << 26 | (false as u32) << 25 | (pre_offset as u32) << 24 | (add_offset as u32) << 23 |
        (transfer_byte as u32) << 22 | (write_back as u32) << 21 | (load as u32) << 20 | base_reg << 16 |
        src_dest_reg << 12 | offset
    }
    
    // LDRB r0, [r0]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, true, true, false, 0, 0, 0),
    R0 = 0xABCDEFD0);
    assert_regs!(cpu.regs, R0 = 0xD0, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r0, [r0]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, true, false, 0, 0, 0),
    R0 = 0xABCDEFD0);
    assert_regs!(cpu.regs, R0 = 0xABCDEFD0, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r1, [r0, #+0x10]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, true, false, 0, 1, 0x10),
    R0 = 0xABCDEFD0);
    assert_regs!(cpu.regs, R0 = 0xABCDEFD0u32, R1 = 0xABCDEFD0u32 + 0x10, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r1, [r0, #+0x10]!
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, true, true, 0, 1, 0x10),
    R0 = 0xABCDEFD0);
    assert_regs!(cpu.regs, R0 = 0xABCDEFD0u32 + 0x10, R1 = 0xABCDEFD0u32 + 0x10, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r1, [r0], #+0x10
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(false, true, false, true, false, 0, 1, 0x10),
    R0 = 0xABCDEFD0);
    assert_regs!(cpu.regs, R0 = 0xABCDEFD0u32 + 0x10, R1 = 0xABCDEFD0u32, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r1, [r0, #-0x4]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, false, false, true, false, 0, 1, 0x4),);
    assert_regs!(cpu.regs, R0 = 0, R1 = 0xFFFFFFFC, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r0, [r15]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, true, false, 15, 0, 0),);
    assert_regs!(cpu.regs, R0 = 0x8, R15 = 4);
    assert_cycle_times(mmu, 1, 1, 1);
    // LDR r15, [r0]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, true, false, 0, 15, 0),
    R0 = 0x100);
    assert_regs!(cpu.regs, R0 = 0x100, R15 = 0x100);
    assert_cycle_times(mmu, 2, 1, 2);

    // STRB r0, [r1]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, true, false, false, 1, 0, 0),
    R0 = 0xFFFF, R1 = 0x100);
    assert_regs!(cpu.regs, R0 = 0xFFFF, R1 = 0x100, R15 = 4);
    assert_writes!(mmu.writes8, 0x100 => 0xFF);
    assert_cycle_times(mmu, 0, 0, 2);
    // STR r0, [r1]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, false, false, 1, 0, 0),
    R0 = 0xABCDEF, R1 = 0x100);
    assert_regs!(cpu.regs, R0 = 0xABCDEF, R1 = 0x100, R15 = 4);
    assert_writes!(mmu.writes32, 0x100 => 0xABCDEF);
    assert_cycle_times(mmu, 0, 0, 2);
    // STR pc, [r1]
    let (cpu, mmu) = run_instr!(single_data_transfer, make_instr(true, true, false, false, false, 1, 15, 0),
    R1 = 0x100);
    assert_regs!(cpu.regs, R1 = 0x100, R15 = 4);
    assert_writes!(mmu.writes32, 0x100 => 0x8);
    assert_cycle_times(mmu, 0, 0, 2);
}
