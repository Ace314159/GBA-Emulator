use super::*;
use crate::cpu::registers::Reg;
use crate::mmu::{Cycle, MemoryHandler};
use std::collections::HashMap;


pub(super) struct TestMMU {
    n_cycle_count: u32,
    s_cycle_count: u32,
    i_cycle_count: u32,
    reading_enabled: bool,
    pub writes8: HashMap<u32, u8>,
    pub writes32: HashMap<u32, u32>,
    pub writes16: HashMap<u32, u16>,
}

impl TestMMU {
    pub fn new() -> TestMMU {
        TestMMU {
            n_cycle_count: 0,
            s_cycle_count: 0,
            i_cycle_count: 0,
            reading_enabled: false,
            writes8: HashMap::new(),
            writes32: HashMap::new(),
            writes16: HashMap::new(),
        }
    }

    pub fn enable_reading(&mut self) {
        self.reading_enabled = true;
    }
}

impl IMMU for TestMMU {
    fn inc_clock(&mut self, cycle_type: Cycle, _addr: u32, _access_width: u32) {
        match cycle_type {
            Cycle::N => self.n_cycle_count += 1,
            Cycle::S => self.s_cycle_count += 1,
            Cycle::I => self.i_cycle_count += 1,
        }
    }
}

impl MemoryHandler for TestMMU {
    fn read8(&self, addr: u32) -> u8 {
        if self.reading_enabled { addr as u8 } else { 0 }
    }

    fn read16(&self, addr: u32) -> u16 {
        if self.reading_enabled { addr as u16 } else { 0 }
    }

    fn read32(&self, addr: u32) -> u32 {
        if self.reading_enabled { addr } else { 0 }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        self.writes8.insert(addr, value);
    }

    fn write16(&mut self, addr: u32, value: u16) {
        self.writes16.insert(addr, value);
    }

    fn write32(&mut self, addr: u32, value: u32) {
        self.writes32.insert(addr, value);
    }
}

macro_rules! run_instr { ($instr_name:ident, $instr:expr, $($reg:ident = $val:expr),*) => { {
    println!("{:08X}", $instr);
    let mut mmu = TestMMU::new();
    let mut cpu = CPU::new(&mut mmu);
    $(
        if Reg::$reg == Reg::CPSR {
            cpu.regs.set_reg(Reg::CPSR, cpu.regs.get_reg(Reg::CPSR) | $val);
        } else {
            cpu.regs.set_reg(Reg::$reg, $val);
        }
    )*
    let instr_len = if cpu.regs.get_t() { cpu.regs.pc = 2; 2 } else { 4 };
    cpu.regs.pc = cpu.regs.pc.wrapping_add(instr_len); // Add instr_len to simulate incrementing pc when fetching instr
    mmu.enable_reading();
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
    if reg_values.get_t() { reg_values.pc = reg_values.pc.wrapping_sub(2) }
    assert_eq!($regs, reg_values);
} } }

macro_rules! assert_writes { ($cpu_writes:expr, $($addr:expr => $val:expr),*) => { {
    let mut writes = HashMap::new();
    $(writes.insert($addr, $val);)*
    assert_eq!($cpu_writes, writes);
} } }

pub(super) fn assert_cycle_times(mmu: TestMMU, s_count: u32, i_count: u32, n_count: u32) {
    assert_eq!(mmu.s_cycle_count, s_count + 2); // 2 extra for initial instr buffer
    assert_eq!(mmu.i_cycle_count, i_count);
    assert_eq!(mmu.n_cycle_count, n_count);
}

#[test]
fn test_shift() {
    fn run_shift(shift_type: u32, operand: u32, shift: u32, immediate: bool, change_status: bool) -> (CPU, u32) {
        let mut mmu = TestMMU::new();
        let mut cpu = CPU::new(&mut mmu);
        let val = cpu.shift(&mut mmu, shift_type, operand, shift, immediate, change_status);
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
pub fn test_mul() {
    fn run_mul(op1: u32, op2: u32) -> (TestMMU, u32) {
        let mut mmu = TestMMU::new();
        let mut cpu = CPU::new(&mut mmu);
        let val = cpu.mul(&mut mmu, op1, op2);
        (mmu, val)
    }

    // 1 I Cycle
    let (mmu, val) = run_mul(0xFFFFFFFF, 0xFFFFFFFF);
    assert_cycle_times(mmu, 0, 1, 0);
    assert_eq!(val, 1);
    let (mmu, val) = run_mul(0, 0xFFFFFF00);
    assert_cycle_times(mmu, 0, 1, 0);
    assert_eq!(val, 0);

    // 2 I Cycles
    let (mmu, val) = run_mul(0xFFFF00FF, 0xFFFF00FF);
    assert_cycle_times(mmu, 0, 2, 0);
    assert_eq!(val, 0xFE02FE01);
    let (mmu, val) = run_mul(0x0000FFFF, 0xFFFF00FF);
    assert_cycle_times(mmu, 0, 2, 0);
    assert_eq!(val, 0xFFFF01);

    // 3 I Cycles
    let (mmu, val) = run_mul(0xFF000000, 0xFF000000);
    assert_cycle_times(mmu, 0, 3, 0);
    assert_eq!(val, 0);
    let (mmu, val) = run_mul(0x00FFFFFF, 0xFF000000);
    assert_cycle_times(mmu, 0, 3, 0);
    assert_eq!(val, 0x1000000);

    // 4 I Cycles
    let (mmu, val) = run_mul(0xF0F0F0F0, 1);
    assert_cycle_times(mmu, 0, 4, 0);
    assert_eq!(val, 0xF0F0F0F0);
}
