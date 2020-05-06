use super::*;
use crate::mmu::MemoryHandler;

struct TestMMU {
    n_cycle_count: u32,
    s_cycle_count: u32,
    i_cycle_count: u32,
}

impl TestMMU {
    pub fn new() -> TestMMU {
        TestMMU {
            n_cycle_count: 0,
            s_cycle_count: 0,
            i_cycle_count: 0,
        }
    }
}

impl IMMU for TestMMU {
    fn inc_clock(&mut self, cycle_count: u32, cycle_type: Cycle, _addr: u32) {
        match cycle_type {
            Cycle::N => self.n_cycle_count += cycle_count,
            Cycle::S => self.s_cycle_count += cycle_count,
            Cycle::I => self.i_cycle_count += cycle_count,
        }
    }
}

impl MemoryHandler for TestMMU {
    fn read8(&self, _addr: u32) -> u8 {
        0
    }

    fn write8(&mut self, _addr: u32, _value: u8) {
        unimplemented!("Test MMU writing not implemented!")
    }
}

macro_rules! run_instr { ($instr_name:ident, $instr:expr, $($reg:ident = $val:expr),*) => { {
    println!("{:08X}", $instr);
    let mut mmu = TestMMU::new();
    let mut cpu = CPU::new(&mut mmu);
    cpu.regs.pc = cpu.regs.pc.wrapping_add(4); // Add 4 to simulate incrementing pc when fetching instr
    mmu.inc_clock(1, Cycle::S, cpu.regs.pc); // Inc to simulate reading instr
    $(
        if Reg::$reg == Reg::CPSR {
            cpu.regs.set_reg(Reg::CPSR, cpu.regs.get_reg(Reg::CPSR) | $val);
        } else {
            cpu.regs.set_reg(Reg::$reg, $val);
        }
    )*
    cpu.$instr_name($instr, &mut mmu);
    (cpu, mmu)
} } }

macro_rules! assert_regs { ($regs:expr, $($reg:ident = $val:expr),*) => { {
    let mut reg_values = RegValues::new();
    $(
        if Reg::$reg == Reg::CPSR {
            reg_values.set_reg(Reg::CPSR, reg_values.get_reg(Reg::CPSR) | $val);
        } else if Reg::$reg == Reg::R15 {
            reg_values.set_reg(Reg::$reg, ($val as u32).wrapping_add(4));
        } else {
            reg_values.set_reg(Reg::$reg, $val);
        }
    )*
    assert_eq!($regs, reg_values);
} } }

fn assert_cycle_times(mmu: TestMMU, s_count: u32, i_count: u32, n_count: u32) {
    assert_eq!(mmu.s_cycle_count, s_count + 1); // 1 extra for initial instr buffer
    assert_eq!(mmu.i_cycle_count, i_count);
    assert_eq!(mmu.n_cycle_count, n_count + 1); // 1 extra for initial instr buffer
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
    // RSCp
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

    // LSL #0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 0, false, 0),
    R0 = 0xA);
    assert_regs!(cpu.regs, R0 = 0xA, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);

    // LSR #0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 1, false, 0),
    R0 = 0xFFFFFFFF);
    assert_regs!(cpu.regs, R0 = 0, R15 = 4, CPSR = 0x60000000);
    assert_cycle_times(mmu, 1, 0, 0);

    // ASR #0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 2, false, 0),
    R0 = 0xFFFFFFFF);
    assert_regs!(cpu.regs, R0 = 0xFFFFFFFF, R15 = 4, CPSR = 0xA0000000);
    assert_cycle_times(mmu, 1, 0, 0);

    // ROR #0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 3, false, 0),
    R0 = 0xFFFFFFFF);
    assert_regs!(cpu.regs, R0 = 0x7FFFFFFF, R15 = 4, CPSR = 0x20000000);
    assert_cycle_times(mmu, 1, 0, 0);

    println!("Third Set");
    // MOV r0, r0 LSL r0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, true, 0, 0, 0, 0, true, 0),);
    assert_regs!(cpu.regs, R15 = 4, CPSR = 0x40000000);

    // MOV pc, r0, LSL r0
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, false, 0, 15, 0, 0, true, 0),
    R0 = 2);
    assert_regs!(cpu.regs, R0 = 2, R15 = 8);
    assert_cycle_times(mmu, 2, 1, 1);

    // MOV r0, pc
    let (cpu, mmu) = run_instr!(data_proc, make_reg_instr(0xD, false, 0, 0, 0, 0, false, 15),);
    assert_regs!(cpu.regs, R0 = 8, R15 = 4);
    assert_cycle_times(mmu, 1, 0, 0);
}

#[test]
// ARM.9: Single Data Transfer (LDR, STR)
fn test_single_data_transfer() {
    
}
