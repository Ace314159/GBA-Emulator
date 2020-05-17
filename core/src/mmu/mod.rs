mod memory;
mod ppu;
mod interrupt_controller;

use memory::ROM;
use memory::RAM;
use ppu::PPU;
use interrupt_controller::InterruptController;

pub struct MMU {
    bios: ROM,
    wram256: RAM,
    wram32: RAM,
    rom: ROM,
    clocks_ahead: u32,

    // IO
    ppu: PPU,
    interrupt_controller: InterruptController,

    // Registers
    haltcnt: u16,
    waitcnt: WaitStateControl,
}

impl MMU {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> MMU {
        MMU {
            bios: ROM::new(0, bios),
            wram256: RAM::new(0x02000000, 0x40000),
            wram32: RAM::new(0x03000000, 0x8000),
            rom: ROM::new(0x08000000, rom),
            clocks_ahead: 0,

            // IO
            ppu: PPU::new(),
            interrupt_controller: InterruptController::new(),

            // Registers
            haltcnt: 0,
            waitcnt: WaitStateControl::new(),
        }
    }
}

impl IMMU for MMU {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32) {
        if cycle_type == Cycle::I { self.clocks_ahead += 1; return }
        self.clocks_ahead += match addr {
            0x00000000 ..= 0x00003FFF => 1, // BIOS ROM
            0x00004000 ..= 0x01FFFFFF => 1, // Unused Memory
            0x02000000 ..= 0x0203FFFF => [3, 3, 6][access_width as usize], // WRAM - On-board 256K
            0x02040000 ..= 0x02FFFFFF => 1, // Unused Memory
            0x03000000 ..= 0x03007FFF => 1, // WRAM - In-chip 32K
            0x03008000 ..= 0x03FFFFFF => 1, // Unused Memory
            0x04000000 ..= 0x040003FE => 1, // IO
            0x04000400 ..= 0x04FFFFFF => 1, // Unused Memory
            0x08000000 ..= 0x09FFFFFF => self.waitcnt.get_access_time(0, cycle_type, access_width),
            0x0A000000 ..= 0x0BFFFFFF => self.waitcnt.get_access_time(1, cycle_type, access_width),
            0x0C000000 ..= 0x0DFFFFFF => self.waitcnt.get_access_time(2, cycle_type, access_width),
            _ => unimplemented!("Clock Cycle for 0x{:08X} not implemented!", addr),
        };
    }
}

impl MemoryHandler for MMU {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.read8(addr),
            0x00004000 ..= 0x01FFFFFF => 0, // Unused Memory
            0x02040000 ..= 0x02FFFFFF => 0, // Unused Memory
            0x03000000 ..= 0x03007FFF => self.wram32.read8(addr),
            0x03008000 ..= 0x03FFFFFF => 0, // Unused Memory
            0x04000000 ..= 0x0400005F => self.ppu.read8(addr),
            0x04000200 => (self.interrupt_controller.enable.read() >> 0) as u8,
            0x04000201 => (self.interrupt_controller.enable.read() >> 8) as u8,
            0x04000202 => (self.interrupt_controller.request.read() >> 0) as u8,
            0x04000203 => (self.interrupt_controller.request.read() >> 8) as u8,
            0x04000204 => (self.waitcnt.read() >> 0) as u8,
            0x04000205 => (self.waitcnt.read() >> 8) as u8,
            0x04000206 ..= 0x04000207 => 0, // Unused IO Register
            0x04000208 => (self.interrupt_controller.master_enable.read() >> 0) as u8,
            0x04000209 => (self.interrupt_controller.master_enable.read() >> 8) as u8,
            0x0400020A ..= 0x040002FF => 0, // Unused IO Register
            0x04000300 => self.haltcnt as u8,
            0x04000301 => (self.haltcnt >> 8) as u8,
            0x04000400 ..= 0x04FFFFFF => 0, // Unused Memory
            0x08000000 ..= 0x0DFFFFFF => self.rom.read8(addr),
            _ => unimplemented!("Memory Handler for 0x{:08X} not implemented!", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x00000000 ..= 0x00003FFF => self.bios.write8(addr, value),
            0x00004000 ..= 0x01FFFFFF => {}, // Unused Memory
            0x02040000 ..= 0x02FFFFFF => {}, // Unused Memory
            0x03000000 ..= 0x03007FFF => self.wram32.write8(addr, value),
            0x03008000 ..= 0x03FFFFFF => {}, // Unused Memory
            0x04000000 ..= 0x0400005F => self.ppu.write8(addr, value),
            0x04000200 => self.interrupt_controller.enable.write( 0x00FF, (value as u16) << 0),
            0x04000201 => self.interrupt_controller.enable.write( 0xFF00, (value as u16) << 8),
            0x04000202 => self.interrupt_controller.request.write( 0x00FF, (value as u16) << 0),
            0x04000203 => self.interrupt_controller.request.write( 0xFF00, (value as u16) << 0),
            0x04000204 => self.waitcnt.write(0x00FF, (value as u16) << 0),
            0x04000205 => self.waitcnt.write(0xFF00, (value as u16) << 8),
            0x04000206 ..= 0x04000207 => {}, // Unused IO Register
            0x04000208 => self.interrupt_controller.master_enable.write(0x00FF, (value as u16) << 0),
            0x04000209 => self.interrupt_controller.master_enable.write(0xFF00, (value as u16) << 8),
            0x0400020A ..= 0x040002FF => {}, // Unused IO Register
            0x04000300 => self.haltcnt = (self.haltcnt & !0x00FF) | value as u16,
            0x04000301 => self.haltcnt = (self.haltcnt & !0xFF00) | (value as u16) << 8,
            0x04000400 ..= 0x04FFFFFF => {}, // Unused Memory
            0x08000000 ..= 0x0DFFFFFF => self.rom.write8(addr, value),
            _ => unimplemented!("Memory Handler for 0x{:08X} not implemented!", addr),
        }
    }
}

pub trait MemoryHandler {
    fn read8(&self, addr: u32) -> u8;
    fn write8(&mut self, addr: u32, value: u8);

    fn read16(&self, addr: u32) -> u16 {
        (self.read8(addr + 0) as u16) << 0 |
        (self.read8(addr + 1) as u16) << 8 
    }
    fn write16(&mut self, addr: u32, value: u16) {
        self.write8(addr + 0, (value >> 0) as u8);
        self.write8(addr + 1, (value >> 8) as u8);
    }

    fn read32(&self, addr: u32) -> u32 {
        (self.read8(addr + 0) as u32) << 0 |
        (self.read8(addr + 1) as u32) << 8 |
        (self.read8(addr + 2) as u32) << 16 |
        (self.read8(addr + 3) as u32) << 24
    }
    fn write32(&mut self, addr: u32, value: u32) {
        self.write8(addr + 0, (value >> 0) as u8);
        self.write8(addr + 1, (value >> 8) as u8);
        self.write8(addr + 2, (value >> 16) as u8);
        self.write8(addr + 3, (value >> 24) as u8);
    }
}

pub trait IMMU: MemoryHandler {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32);
}

pub trait IORegister {
    fn read(&self) -> u16;
    fn write(&mut self, mask: u16, value: u16);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cycle {
    S = 0,
    N = 1,
    I,
    // C - No coprocessor in GBA
}

struct WaitStateControl {
    sram: usize,
    wait_states: [[usize; 2]; 3],
    phi_terminal_out: usize,
    prefetch_buffer: bool,
    type_flag: bool,
}

impl WaitStateControl {
    const ACCESS_TIMINGS: [[[u32; 4]; 2]; 3] = [
        [[4, 3, 2, 8], [2, 1, 0, 0]],
        [[4, 3, 2, 8], [4, 1, 0, 0]],
        [[4, 3, 2, 8], [8, 1, 0, 0]]
    ];

    pub fn new() -> WaitStateControl {
        WaitStateControl {
            sram: 0,
            wait_states: [[0; 2]; 3],
            phi_terminal_out: 0,
            prefetch_buffer: false,
            type_flag: false,
        }
    }

    pub fn get_access_time(&self, wait_state: usize, cycle_type: Cycle, access_len: u32) -> u32 {
        assert_ne!(cycle_type, Cycle::I);
        assert_eq!(access_len <= 2, true);
        let wait_state_setting = self.wait_states[wait_state][cycle_type as usize];
        WaitStateControl::ACCESS_TIMINGS[wait_state][cycle_type as usize][wait_state_setting] + if access_len < 2 { 0 }
        else {
            WaitStateControl::ACCESS_TIMINGS[wait_state][Cycle::S as usize]
                [self.wait_states[wait_state][Cycle::S as usize]]
            }
    }
}

impl IORegister for WaitStateControl {
    fn read(&self) -> u16 {
        ((self.type_flag as usize) << 15 | (self.prefetch_buffer as usize) << 14 | self.phi_terminal_out << 11 |
        self.wait_states[2][1] << 10 | self.wait_states[2][0] << 8 | self.wait_states[1][1] << 7 |
        self.wait_states[1][0] << 5 | self.wait_states[0][1] << 4 | self.wait_states[0][0] << 2 | self.sram) as u16
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = (value & mask) as usize;
        self.sram = value & 0x3;
        self.wait_states[0][0] = (value >> 2) & 0x3;
        self.wait_states[0][1] = (value >> 4) & 0x1;
        self.wait_states[1][0] = (value >> 5) & 0x3;
        self.wait_states[1][1] = (value >> 7) & 0x1;
        self.wait_states[2][0] = (value >> 8) & 0x3;
        self.wait_states[2][1] = (value >> 10) & 0x1;
        self.phi_terminal_out = (value >> 11) & 0x3;
        self.prefetch_buffer = (value >> 14) & 0x1 != 0;
        // Type Flag is read only
    }
}
