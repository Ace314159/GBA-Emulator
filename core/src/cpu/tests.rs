use super::*;
use crate::mmu::Cycle;
use crate::mmu::MemoryHandler;
use std::collections::HashMap;


pub(super) struct TestMMU {
    n_cycle_count: u32,
    s_cycle_count: u32,
    i_cycle_count: u32,
    reading_enabled: bool,
    pub writes8: HashMap<u32, u8>,
    pub writes32: HashMap<u32, u32>,
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

    fn read32(&self, addr: u32) -> u32 {
        if self.reading_enabled { addr } else { 0 }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        self.writes8.insert(addr, value);
    }

    fn write32(&mut self, addr: u32, value: u32) {
        self.writes32.insert(addr, value);
    }
}

macro_rules! run_instr { ($instr_name:ident, $instr:expr, $($reg:ident = $val:expr),*) => { {
    println!("{:08X}", $instr);
    let mut mmu = TestMMU::new();
    let mut cpu = CPU::new(&mut mmu);
    cpu.regs.pc = cpu.regs.pc.wrapping_add(4); // Add 4 to simulate incrementing pc when fetching instr
    $(
        if Reg::$reg == Reg::CPSR {
            cpu.regs.set_reg(Reg::CPSR, cpu.regs.get_reg(Reg::CPSR) | $val);
        } else {
            cpu.regs.set_reg(Reg::$reg, $val);
        }
    )*
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