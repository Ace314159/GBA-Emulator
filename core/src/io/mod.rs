mod memory;
mod ppu;
mod apu;
mod dma;
mod timers;
pub mod keypad;
mod interrupt_controller;

use std::sync::{Arc, Mutex};
use flume::Sender;

use memory::MemoryHandler;
use dma::DMA;
use timers::*;
use ppu::PPU;
use apu::APU;
use keypad::Keypad;
use interrupt_controller::{InterruptController, InterruptRequest};
use crate::gba::VisibleMemoryRegion;

pub struct IO {
    bios: Vec<u8>,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    rom: Vec<u8>,
    cart_ram: Vec<u8>,
    clocks_ahead: u32,

    // IO
    ppu: PPU,
    apu: APU,
    dma: DMA,
    timers: Timers,
    keypad: Keypad,
    interrupt_controller: InterruptController,

    // Registers
    haltcnt: u16,
    waitcnt: WaitStateControl,

    mgba_test_suite: MGBATestSuite,
}

impl IO {
    const EWRAM_MASK: u32 = 0x3FFFF;
    const IWRAM_MASK: u32 = 0x7FFF;

    pub fn new(bios: Vec<u8>, rom: Vec<u8>, tx: Sender<bool>) -> (IO, Arc<Mutex<Vec<u16>>>) {
        let (ppu, pixels) = PPU::new(tx);
        (IO {
            bios,
            ewram: vec![0; 0x40000],
            iwram: vec![0; 0x8000],
            rom,
            cart_ram: vec![0xFF; 0x10000],
            clocks_ahead: 0,

            // IO
            ppu,
            apu: APU::new(),
            dma: DMA::new(),
            timers: Timers::new(),
            keypad: Keypad::new(),
            interrupt_controller: InterruptController::new(),

            // Registers
            haltcnt: 0,
            waitcnt: WaitStateControl::new(),

            mgba_test_suite: MGBATestSuite::new(),
        }, pixels)
    }

    pub fn render_map(&self, bg_i: usize) -> (Vec<u16>, usize, usize) {
        self.ppu.render_map(bg_i)
    }

    pub fn render_tiles(&self, palette: usize, block: usize, bpp8: bool) -> (Vec<u16>, usize, usize) {
        self.ppu.render_tiles(palette, block, bpp8)
    }

    pub fn render_palettes(&self) -> (Vec<u16>, usize, usize) {
        self.ppu.render_palettes()
    }

    pub fn peek_mem(&self, region: VisibleMemoryRegion, addr: u32) -> u8 {
        self.read::<u8>(region.get_start_addr() + addr)
    }

    pub fn press_key(&mut self, key: keypad::KEYINPUT) {
        self.keypad.press_key(key);
    }

    pub fn release_key(&mut self, key: keypad::KEYINPUT) {
        self.keypad.release_key(key);
    }

    pub fn run_dmas(&mut self) {
        let dma_channel = self.dma.get_channel_running(self.ppu.hblank_called(), self.ppu.vblank_called());
        if dma_channel < 4 {
            let channel = &mut self.dma.channels[dma_channel];
            let count = channel.count_latch;
            let mut src_addr = channel.sad_latch;
            let mut dest_addr = channel.dad_latch;
            let src_addr_ctrl = channel.cnt.src_addr_ctrl;
            let dest_addr_ctrl = channel.cnt.dest_addr_ctrl;
            let transfer_32 = channel.cnt.transfer_32;
            let irq = channel.cnt.irq;
            channel.cnt.enable = channel.cnt.start_timing != 0 && channel.cnt.repeat;
            info!("Running DMA{}: Writing to {:08X} from {:08X}, size: {}", dma_channel, dest_addr,
            src_addr, if transfer_32 { 32 } else { 16 });

            let access_width = if transfer_32 { 2 } else { 1 };
            let addr_change = if transfer_32 { 4 } else { 2 };
            let mut first = true;
            let original_dest_addr = dest_addr;
            for _ in 0..count {
                let cycle_type = if first { Cycle::N } else { Cycle::S };
                self.inc_clock(cycle_type, src_addr, access_width);
                self.inc_clock(cycle_type, dest_addr, access_width);
                if transfer_32 { self.write::<u32>(dest_addr, self.read::<u32>(src_addr)) }
                else { self.write::<u16>(dest_addr, self.read::<u16>(src_addr)) }

                src_addr = match src_addr_ctrl {
                    0 => src_addr.wrapping_add(addr_change),
                    1 => src_addr.wrapping_sub(addr_change),
                    2 => src_addr,
                    _ => panic!("Invalid DMA Source Address Control!"),
                };
                dest_addr = match dest_addr_ctrl {
                    0 | 3 => dest_addr.wrapping_add(addr_change),
                    1 => dest_addr.wrapping_sub(addr_change),
                    2 => dest_addr,
                    _ => unreachable!(),
                };
                first = false;
            }
            let channel = &mut self.dma.channels[dma_channel];
            channel.sad_latch = src_addr;
            channel.dad_latch = dest_addr;
            if channel.cnt.enable { channel.count_latch = channel.count.count as u32 } // Only reload Count
            if dest_addr_ctrl == 3 { channel.dad_latch = original_dest_addr }
            for _ in 0..2 { self.inc_clock(Cycle::I, 0, 0) }

            
            if irq { self.interrupt_controller.request |= match dma_channel {
                0 => InterruptRequest::DMA0,
                1 => InterruptRequest::DMA1,
                2 => InterruptRequest::DMA2,
                3 => InterruptRequest::DMA3,
                _ => unreachable!(),
            } }
        }
    }
}

impl IIO for IO {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32) {
        let clocks_inc = if cycle_type == Cycle::I { 1 }
        else { match addr {
            0x00000000 ..= 0x00003FFF => 1, // BIOS ROM
            0x00004000 ..= 0x01FFFFFF => 1, // Unused Memory
            0x02000000 ..= 0x0203FFFF => [3, 3, 6][access_width as usize], // WRAM - On-board 256K
            0x02040000 ..= 0x02FFFFFF => 1, // Unused Memory
            0x03000000 ..= 0x03007FFF => 1, // WRAM - In-chip 32K
            0x03008000 ..= 0x03FFFFFF => 1, // Unused Memory
            0x04000000 ..= 0x040003FE => 1, // IO
            0x04000400 ..= 0x04FFFFFF => 1, // Unused Memory
            0x05000000 ..= 0x05FFFFFF => if access_width < 2 { 1 } else { 2 }, // Palette RAM
            0x06000000 ..= 0x06FFFFFF => if access_width < 2 { 1 } else { 2 }, // VRAM
            0x07000000 ..= 0x07FFFFFF => 1, // OAM
            0x08000000 ..= 0x09FFFFFF => self.waitcnt.get_access_time(0, cycle_type, access_width),
            0x0A000000 ..= 0x0BFFFFFF => self.waitcnt.get_access_time(1, cycle_type, access_width),
            0x0C000000 ..= 0x0DFFFFFF => self.waitcnt.get_access_time(2, cycle_type, access_width),
            0x0E000000 ..= 0x0E00FFFF => 1,
            0x0E010000 ..= 0x0FFFFFFF => 1,
            _ if addr & 0xF0000000 != 0 => 1,
            _ => unimplemented!("Clock Cycle for 0x{:08X} not implemented!", addr),
        }};

        for _ in 0..clocks_inc { self.interrupt_controller.request |= self.timers.clock() }
        self.clocks_ahead += clocks_inc;
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

pub trait IIO: MemoryHandler {
    fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32);
    fn interrupts_requested(&mut self) -> bool;
}

pub trait IORegister {
    fn read(&self, byte: u8) -> u8;
    fn write(&mut self, byte: u8, value: u8);
}

pub trait IORegisterController {
    fn read_register(&self, addr: u32) -> u8;
    fn write_register(&mut self, addr: u32, value: u8);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cycle {
    S = 0,
    N = 1,
    I,
    // C - No coprocessor in GBA
}

struct WaitStateControl {
    cart_ram: usize,
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
            cart_ram: 0,
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
                    self.wait_states[0][1] << 4 |self.wait_states[0][0] << 2 | self.cart_ram) as u8,
            1 => ((self.type_flag as usize) << 7 | (self.prefetch_buffer as usize) << 6 | self.phi_terminal_out << 3 |
                self.wait_states[2][1] << 2 | self.wait_states[2][0]) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                let value = value as usize;
                self.cart_ram = value & 0x3;
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
            _ => unreachable!(),
        }
    }
}


enum MGBALogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug
}

impl MGBALogLevel {
    pub fn new(val: u16) -> Self {
        use MGBALogLevel::*;
        match val {
            0 => Fatal,
            1 => Error,
            2 => Warn,
            3 => Info,
            4 => Debug,
            _ => panic!("Invalid mGBA Log Level!"),
        }
    }
}

struct MGBATestSuite {
    buffer: [char; 0x100],
    // Registers
    enable: u16,
    flags: u16,
}

impl MGBATestSuite {
    pub fn new() -> MGBATestSuite {
        MGBATestSuite {
            buffer: ['\0'; 0x100],
            enable: 0,
            flags: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        false //self.enable == 0xC0DE
    }

    pub fn write_enable(&mut self, addr: u32, value: u8) {
        match addr {
            0x4FFF780 => self.enable = self.enable & !0x00FF | (value as u16) << 0 & 0x00FF,
            0x4FFF781 => self.enable = self.enable & !0xFF00 | (value as u16) << 8 & 0xFF00,
            _ => (),
        }
    }

    pub fn read_register(&self, addr: u32) -> u8 {
        match addr {
            0x4FFF780 => if self.enabled() { 0xEA } else { 0 },
            0x4FFF781 => if self.enabled() { 0x1D } else { 0 },
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        if !self.enabled() { return }
        match addr {
            0x4FFF600 ..= 0x4FFF6FF => self.buffer[(addr - 0x4FFF600) as usize] = value as char,
            0x4FFF700 => self.flags = self.flags & !0x00FF | (value as u16) << 0 & 0x00FF,
            0x4FFF701 => {
                self.flags = self.flags & !0xFF00 | (value as u16) << 8 & 0xFF00;
                if self.flags & 0x100 != 0 {
                    use MGBALogLevel::*;
                    let str: String = self.buffer.iter().collect();
                    match MGBALogLevel::new(self.flags & 0x7) {
                        Fatal => print!("{}", str),
                        Error => print!("{}", str),
                        Warn => print!("{}", str),
                        Info => print!("{}", str),
                        Debug => print!("{}", str),
                    }
                }
            },
            _ => (), 
        }
    }
}
