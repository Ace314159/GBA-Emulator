mod memory;
mod ppu;
mod dma;
pub mod keypad;
mod interrupt_controller;

use crate::gba::Display;
use memory::ROM;
use memory::RAM;
use dma::DMA;
use ppu::PPU;
use keypad::Keypad;
use interrupt_controller::{InterruptController, InterruptRequest};

pub struct IO {
    bios: ROM,
    wram256: RAM,
    wram32: RAM,
    rom: ROM,
    sram: RAM,
    clocks_ahead: u32,

    // IO
    ppu: PPU,
    dma: DMA,
    keypad: Keypad,
    interrupt_controller: InterruptController,

    // Registers
    haltcnt: u16,
    waitcnt: WaitStateControl,

    p: bool,
}

impl IO {
    pub fn new(bios: Vec<u8>, rom: Vec<u8>) -> IO {
        IO {
            bios: ROM::new(0, bios),
            wram256: RAM::new(0x02000000, 0x40000),
            wram32: RAM::new(0x03000000, 0x8000),
            rom: ROM::new(0x08000000, rom),
            sram: RAM::new(0x0E000000, 0x10000),
            clocks_ahead: 0,

            // IO
            ppu: PPU::new(),
            dma: DMA::new(),
            keypad: Keypad::new(),
            interrupt_controller: InterruptController::new(),

            // Registers
            haltcnt: 0,
            waitcnt: WaitStateControl::new(),

            p: false,
        }
    }

    pub fn needs_to_render(&mut self) -> bool {
        let needs_to_render = self.ppu.needs_to_render;
        self.ppu.needs_to_render = false;
        needs_to_render
    }

    pub fn get_pixels(&self) -> &[u16; Display::WIDTH * Display::HEIGHT] {
        &self.ppu.pixels
    }

    pub fn press_key(&mut self, key: keypad::KEYINPUT) {
        self.keypad.press_key(key);
    }

    pub fn release_key(&mut self, key: keypad::KEYINPUT) {
        self.keypad.release_key(key);
    }

    pub fn run_dmas(&mut self) {
        let dma_channel = self.dma.get_channel_running();
        if dma_channel < 4 {
            let channel = &mut self.dma.channels[dma_channel];
            let count = channel.count.count;
            if count == 0 { return }
            let mut src_addr = channel.sad.addr;
            let mut dest_addr = channel.dad.addr;
            let src_addr_ctrl = channel.cnt.src_addr_ctrl;
            let dest_addr_ctrl = channel.cnt.dest_addr_ctrl;
            let transfer_32 = channel.cnt.transfer_32;
            let irq = channel.cnt.irq;
            channel.cnt.enable = channel.cnt.repeat;

            let access_width = if transfer_32 { 2 } else { 1 };
            let addr_change = if transfer_32 { 4 } else { 2 };
            let mut first = true;
            for _ in 0..count {
                let cycle_type = if first { Cycle::N } else { Cycle::S };
                self.inc_clock(cycle_type, src_addr, access_width);
                self.inc_clock(cycle_type, dest_addr, access_width);
                if transfer_32 { self.write32(dest_addr, self.read32(src_addr)) }
                else { self.write16(dest_addr, self.read16(src_addr)) }

                src_addr = match src_addr_ctrl {
                    0 | 3 => src_addr.wrapping_add(addr_change),
                    1 => src_addr.wrapping_sub(addr_change),
                    2 => src_addr,
                    _ => panic!("Invalid DMA Source Address Control!"),
                };
                dest_addr = match dest_addr_ctrl {
                    0 => dest_addr.wrapping_add(addr_change),
                    1 => dest_addr.wrapping_sub(addr_change),
                    2 => dest_addr,
                    _ => panic!("Invalid DMA Source Address Control!"),
                };
                first = false;
            }
            for _ in 0..2 { self.inc_clock(Cycle::I, 0, 0) }

            
            if irq { self.interrupt_controller.request |= match dma_channel {
                0 => InterruptRequest::DMA0,
                1 => InterruptRequest::DMA1,
                2 => InterruptRequest::DMA2,
                3 => InterruptRequest::DMA3,
                _ => panic!("Invalid DMA Channel!"),
            } }
        }
    }
}

impl IIO for IO {
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
            0x05000000 ..= 0x050003FF => if access_width < 2 { 1 } else { 2 }, // Palette RAM
            0x06000000 ..= 0x06017FFF => if access_width < 2 { 1 } else { 2 }, // VRAM
            0x07000000 ..= 0x070003FF => 1, // OAM
            0x08000000 ..= 0x09FFFFFF => self.waitcnt.get_access_time(0, cycle_type, access_width),
            0x0A000000 ..= 0x0BFFFFFF => self.waitcnt.get_access_time(1, cycle_type, access_width),
            0x0C000000 ..= 0x0DFFFFFF => self.waitcnt.get_access_time(2, cycle_type, access_width),
            0x0E000000 ..= 0x0E00FFFF => 1,
            _ => unimplemented!("Clock Cycle for 0x{:08X} not implemented!", addr),
        };

        while self.clocks_ahead >= 4 {
            self.clocks_ahead -= 4;
            self.interrupt_controller.request |= self.ppu.emulate_dot();
        }
    }

    fn interrupts_requested(&mut self) -> bool {
        if self.keypad.interrupt_requested() { self.interrupt_controller.request |= InterruptRequest::KEYPAD }

        self.interrupt_controller.master_enable.bits() != 0 &&
        (self.interrupt_controller.request.bits() & self.interrupt_controller.enable.bits()) != 0
    }
}

impl MemoryHandler for IO {
    fn read8(&self, addr: u32) -> u8 {
        match MemoryRegion::get_region(addr) {
            MemoryRegion::BIOS => self.bios.read8(addr),
            MemoryRegion::WRAM256 => self.wram256.read8(addr & 0xFF03FFFF),
            MemoryRegion::WRAM32 => self.wram32.read8(addr & 0xFF007FFF),
            MemoryRegion::IO => match addr {
                0x04000000 ..= 0x0400005F => self.ppu.read8(addr),
                0x040000B0 ..= 0x040000DF => self.dma.read8(addr),
                0x04000130 => self.keypad.keyinput.read(0),
                0x04000131 => self.keypad.keyinput.read(1),
                0x04000132 => self.keypad.keycnt.read(0),
                0x04000133 => self.keypad.keycnt.read(1),
                0x04000200 => self.interrupt_controller.enable.read(0),
                0x04000201 => self.interrupt_controller.enable.read(1),
                0x04000202 => self.interrupt_controller.request.read(0),
                0x04000203 => self.interrupt_controller.request.read(1),
                0x04000204 => self.waitcnt.read(0),
                0x04000205 => self.waitcnt.read(1),
                0x04000206 ..= 0x04000207 => 0, // Unused IO Register
                0x04000208 => self.interrupt_controller.master_enable.read(0),
                0x04000209 => self.interrupt_controller.master_enable.read(1),
                0x0400020A ..= 0x040002FF => 0, // Unused IO Register
                0x04000089 => 0x2,
                0x04000300 => self.haltcnt as u8,
                0x04000301 => (self.haltcnt >> 8) as u8,
                
                _ => { if self.p { println!("Reading Unimplemented IO Register at {:08X}", addr) } 0 }
            },
            MemoryRegion::PALETTE => self.ppu.read_palette_ram(addr),
            MemoryRegion::VRAM => self.ppu.read_vram(addr),
            MemoryRegion::OAM => self.ppu.read_oam(addr),
            MemoryRegion::ROM => self.rom.read8(addr),
            MemoryRegion::SRAM => self.sram.read8(addr),
            MemoryRegion::UNUSED => { if self.p { println!("Reading Unused Memory at {:08X}", addr) } 0 }
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match MemoryRegion::get_region(addr) {
            MemoryRegion::BIOS => self.bios.write8(addr, value),
            MemoryRegion::WRAM256 => self.wram256.write8(addr & 0xFF03FFFF, value),
            MemoryRegion::WRAM32 => self.wram32.write8(addr & 0xFF007FFF, value),
            MemoryRegion::IO => match addr {
                0x04000000 ..= 0x0400005F => self.ppu.write8(addr, value),
                0x040000B0 ..= 0x040000DF => self.dma.write8(addr, value),
                0x04000130 => self.keypad.keyinput.write(0, value),
                0x04000131 => self.keypad.keyinput.write(1, value),
                0x04000132 => self.keypad.keycnt.write(0, value),
                0x04000133 => self.keypad.keycnt.write(1, value),
                0x04000200 => self.interrupt_controller.enable.write(0, value),
                0x04000201 => self.interrupt_controller.enable.write(1, value),
                0x04000202 => self.interrupt_controller.request.write(0, value),
                0x04000203 => self.interrupt_controller.request.write(1, value),
                0x04000204 => self.waitcnt.write(0, value),
                0x04000205 => self.waitcnt.write(1, value),
                0x04000206 ..= 0x04000207 => {}, // Unused IO Register
                0x04000208 => self.interrupt_controller.master_enable.write(0, value),
                0x04000209 => self.interrupt_controller.master_enable.write(1, value),
                0x0400020A ..= 0x040002FF => {}, // Unused IO Register
                0x04000300 => self.haltcnt = (self.haltcnt & !0x00FF) | value as u16,
                0x04000301 => self.haltcnt = (self.haltcnt & !0xFF00) | (value as u16) << 8,
                _ => if self.p { println!("Writng Unimplemented IO Register at {:08X} = {:08X}", addr, value) }
            },
            MemoryRegion::PALETTE => self.ppu.write_palette_ram(addr, value),
            MemoryRegion::VRAM => self.ppu.write_vram(addr, value),
            MemoryRegion::OAM => self.ppu.write_oam(addr, value),
            MemoryRegion::ROM => self.rom.write8(addr, value),
            MemoryRegion::SRAM => self.sram.write8(addr, value),
            MemoryRegion::UNUSED => if self.p { println!("Writng Unused Memory at {:08X} {:08X}", addr, value) }
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

pub enum MemoryRegion {
    BIOS,
    WRAM256,
    WRAM32,
    IO,
    PALETTE,
    VRAM,
    OAM,
    ROM,
    SRAM,
    UNUSED,
}

impl MemoryRegion {
    pub fn get_region(addr: u32) -> MemoryRegion {
        match addr >> 24 {
            0x00 if addr < 0x00004000 => MemoryRegion::BIOS, // Not Mirrored
            0x02 => MemoryRegion::WRAM256,
            0x03 => MemoryRegion::WRAM32,
            0x04 => MemoryRegion::IO,
            0x05 => MemoryRegion::PALETTE,
            0x06 => MemoryRegion::VRAM,
            0x07 => MemoryRegion::OAM,
            0x08 ..= 0x0D => MemoryRegion::ROM,
            0x0E => MemoryRegion::SRAM,
            _ => MemoryRegion::UNUSED,
        }
    }
}

pub trait IIO: MemoryHandler {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32);
    fn interrupts_requested(&mut self) -> bool;
}

pub trait IORegister {
    fn read(&self, byte: u8) -> u8;
    fn write(&mut self, byte: u8, value: u8);
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
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.wait_states[1][1] << 7 | self.wait_states[1][0] << 5 |
                    self.wait_states[0][1] << 4 |self.wait_states[0][0] << 2 | self.sram) as u8,
            1 => ((self.type_flag as usize) << 7 | (self.prefetch_buffer as usize) << 6 | self.phi_terminal_out << 3 |
                self.wait_states[2][1] << 2 | self.wait_states[2][0]) as u8,
            _ => panic!("Invalid Byte!!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                let value = value as usize;
                self.sram = value & 0x3;
                self.wait_states[0][0] = (value >> 2) & 0x3;
                self.wait_states[0][1] = (value >> 4) & 0x1;
                self.wait_states[1][0] = (value >> 5) & 0x3;
                self.wait_states[1][1] = (value >> 7) & 0x1;
            },
            1 => {
                let value = value as usize;
                self.wait_states[2][0] = (value >> 0) & 0x3;
                self.wait_states[2][1] = (value >> 2) & 0x1;
                self.phi_terminal_out = (value >> 3) & 0x3;
                self.prefetch_buffer = (value >> 6) & 0x1 != 0;
                // Type Flag is read only
            }
            _ => panic!("Invalid Byte!"),
        }
    }
}
