mod memory;
mod scheduler;
mod ppu;
mod apu;
mod dma;
mod timers;
pub mod keypad;
mod interrupt_controller;
mod gpio;
mod cart_backup;

use std::cell::Cell;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use flume::{Receiver, Sender};

pub use memory::{MemoryHandler, MemoryRegion, MemoryValue};
use scheduler::{Event, EventType, Scheduler};
use dma::DMA;
use timers::Timers;
use ppu::PPU;
use apu::APU;
use keypad::{Keypad, KEYINPUT};
use interrupt_controller::{InterruptController, InterruptRequest};
use gpio::{GPIO, RTC};
use cart_backup::CartBackup;

use crate::gba::VisibleMemoryRegion;
pub use ppu::{DebugSpecification, DebugWindows};

pub struct IO {
    bios: Vec<u8>,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    rom: Vec<u8>,
    scheduler: Scheduler,
    clocks_ahead: u32,

    // IO
    ppu: PPU,
    apu: APU,
    dma: DMA,
    timers: Timers,
    keypad: Keypad,
    interrupt_controller: InterruptController,
    rtc: RTC,
    cart_backup: Box<dyn CartBackup>,

    // Registers
    haltcnt: u16,
    waitcnt: WaitStateControl,

    // Open Bus
    pc: u32,
    in_thumb: bool,
    instr_buffer: [u32; 2],
    bios_latch: Cell<u32>,

    mgba_test_suite: mgba_test_suite::MGBATestSuite,
}

impl IO {
    const EWRAM_MASK: u32 = 0x3FFFF;
    const IWRAM_MASK: u32 = 0x7FFF;

    pub fn new(bios: Vec<u8>, rom_file: PathBuf, render_tx: Sender<DebugWindows>, keypad_rx: Receiver<(KEYINPUT, bool)>) ->
        (IO, Arc<Mutex<Vec<u16>>>, Arc<Mutex<DebugSpecification>>) {
        let (ppu, pixels, debug_windows_spec) = PPU::new(render_tx);
        assert_eq!(rom_file.extension().unwrap(), "gba");
        let mut save_file = rom_file.clone();
        save_file.set_extension("sav");
        let rom = fs::read(rom_file).unwrap();
        let cart_backup = CartBackup::get(&rom, save_file);
        let rtc = RTC::new(&rom);
        (IO {
            bios,
            ewram: vec![0; 0x40000],
            iwram: vec![0; 0x8000],
            rom,
            scheduler: Scheduler::new(),
            clocks_ahead: 0,

            // IO
            ppu,
            apu: APU::new(),
            dma: DMA::new(),
            timers: Timers::new(),
            keypad: Keypad::new(keypad_rx),
            interrupt_controller: InterruptController::new(),
            rtc,
            cart_backup,

            // Registers
            haltcnt: 0,
            waitcnt: WaitStateControl::new(),

            // Open Bus
            pc: 0,
            in_thumb: false,
            instr_buffer: [0; 2],
            bios_latch: Cell::new(0),

            mgba_test_suite: mgba_test_suite::MGBATestSuite::new(),
        }, pixels, debug_windows_spec)
    }

    pub fn inc_clock(&mut self, cycle_type: Cycle, addr: u32, access_width: u32) {
        let clocks_inc = if cycle_type == Cycle::I { 1 }
        else { match MemoryRegion::get_region(addr) {
            MemoryRegion::BIOS => 1, // BIOS ROM
            MemoryRegion::EWRAM => [3, 3, 6][access_width as usize], // WRAM - On-board 256K
            MemoryRegion::IWRAM => 1,
            MemoryRegion::IO => 1,
            MemoryRegion::Palette => if access_width < 2 { 1 } else { 2 },
            MemoryRegion::VRAM => if access_width < 2 { 1 } else { 2 },
            MemoryRegion::OAM => 1,
            MemoryRegion::ROM0L | MemoryRegion::ROM0H =>
                self.waitcnt.get_rom_access_time(0, cycle_type, access_width, addr),
            MemoryRegion::ROM1L | MemoryRegion::ROM1H =>
                self.waitcnt.get_rom_access_time(1, cycle_type, access_width, addr),
            MemoryRegion::ROM2L | MemoryRegion::ROM2H =>
                self.waitcnt.get_rom_access_time(2, cycle_type, access_width, addr),
            MemoryRegion::SRAM => self.waitcnt.get_sram_access_time(cycle_type),
            MemoryRegion::Unused => 1,
        }};
        self.waitcnt.clock_prefetch(clocks_inc);

        for _ in 0..clocks_inc {
            self.handle_events();
            self.rtc.clock();
            self.apu.clock();
        }
        self.clocks_ahead += clocks_inc;
        while self.clocks_ahead >= 4 {
            self.clocks_ahead -= 4;
            self.interrupt_controller.request |= self.ppu.emulate_dot();
        }
    }

    pub fn interrupts_requested(&mut self) -> bool {
        if self.keypad.interrupt_requested() { self.interrupt_controller.request |= InterruptRequest::KEYPAD }

        self.interrupt_controller.master_enable.bits() != 0 &&
        (self.interrupt_controller.request.bits() & self.interrupt_controller.enable.bits()) != 0
    }

    pub fn peek_mem(&self, region: VisibleMemoryRegion, addr: u32) -> u8 {
        self.read::<u8>(region.get_start_addr() + addr)
    }

    pub fn get_cycle(&self) -> usize { self.scheduler.cycle }

    pub fn poll_keypad_updates(&mut self) {
        if self.ppu.rendered_frame() {
            self.cart_backup.save_to_file();
            self.keypad.poll();
        }
    }

    pub fn run_dma(&mut self) {
        let dma_channel = self.dma.get_channel_running(
            self.ppu.hblank_called(), self.ppu.vblank_called(), [self.apu.fifo_a_req(), self.apu.fifo_b_req()]
        );
        if dma_channel < 4 {
            self.dma.in_dma = true;
            let channel = &mut self.dma.channels[dma_channel];
            let is_fifo = (channel.num == 1 || channel.num == 2) && channel.cnt.start_timing == 3;
            let count = if is_fifo { 4 } else { channel.count_latch };
            let mut src_addr = channel.sad_latch;
            let mut dest_addr = channel.dad_latch;
            let src_addr_ctrl = channel.cnt.src_addr_ctrl;
            let dest_addr_ctrl = if is_fifo { 2 } else { channel.cnt.dest_addr_ctrl };
            let transfer_32 = if is_fifo { true } else { channel.cnt.transfer_32 };
            let irq = channel.cnt.irq;
            channel.cnt.enable = channel.cnt.start_timing != 0 && channel.cnt.repeat;
            info!("Running DMA{}: Writing {} values to {:08X} from {:08X}, size: {}", dma_channel, count, dest_addr,
            src_addr, if transfer_32 { 32 } else { 16 });
            if MemoryRegion::get_region(dest_addr) == MemoryRegion::ROM2H &&
            self.cart_backup.is_eeprom_access(dest_addr, self.rom.len()) {
                self.cart_backup.init_eeprom(count)
            }

            let (access_width, addr_change, addr_mask) = if transfer_32 { (2, 4, 0x3) } else { (1, 2, 0x1) };
            src_addr &= !addr_mask;
            dest_addr &= !addr_mask;
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
            self.dma.in_dma = false;
        }
    }
}

pub trait IORegister {
    fn read(&self, byte: u8) -> u8;
    fn write(&mut self, scheduler: &mut Scheduler, byte: u8, value: u8);
}

pub trait IORegisterController {
    fn read_register(&self, addr: u32) -> u8;
    fn write_register(&mut self, addr: u32, value: u8);
}

#[derive(Clone, Copy)]
pub enum AccessType {
    N,
    S,
}

impl std::convert::Into<Cycle> for AccessType {
    fn into(self) -> Cycle {
        match self {
            AccessType::N => Cycle::N,
            AccessType::S => Cycle::S,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cycle {
    N,
    S,
    I,
    // C - No coprocessor in GBA
}

struct WaitStateControl {
    sram_setting: usize,
    n_wait_state_settings: [usize; 3],
    s_wait_state_settings: [usize; 3],
    phi_terminal_out: usize,
    use_prefetch: bool,
    type_flag: bool,
    // Prefetch Buffer
    can_prefetch: bool,
    prefetch: VecDeque<u32>,
    prefetch_waitstate: usize,
    prefetch_addr: u32,
    prefetch_cycles_spent: u32,
}

impl WaitStateControl {
    const N_ACCESS_TIMINGS: [u32; 4] = [4, 3, 2, 8];
    const S_ACCESS_TIMINGS: [[u32; 2]; 3] = [
        [2, 1],
        [4, 1],
        [8, 1],
    ];
    const SRAM_ACCESS_TIMINGS: [u32; 4] = [4, 3, 2, 8];

    pub fn new() -> WaitStateControl {
        WaitStateControl {
            sram_setting: 0,
            n_wait_state_settings: [0; 3],
            s_wait_state_settings: [0; 3],
            phi_terminal_out: 0,
            use_prefetch: false,
            type_flag: false,
            // Prefetch Buffer
            can_prefetch: true,
            prefetch: VecDeque::new(),
            prefetch_waitstate: 0,
            prefetch_addr: 0x08000000,
            prefetch_cycles_spent: 0,
        }
    }

    pub fn get_rom_access_time(&mut self, wait_state: usize, cycle_type: Cycle, access_len: u32, addr: u32) -> u32 {
        assert_ne!(cycle_type, Cycle::I);
        assert!(access_len <= 2);
        let default_stall_time = match cycle_type {
            Cycle::N => {
                WaitStateControl::N_ACCESS_TIMINGS[self.n_wait_state_settings[wait_state]]
            },
            Cycle::S => {
                WaitStateControl::S_ACCESS_TIMINGS[wait_state][self.s_wait_state_settings[wait_state]]
            },
            Cycle::I => unreachable!(),
        };
        self.can_prefetch = false;
        let addr = addr & !0x1;
        let stall_time = if self.use_prefetch {
            if self.prefetch.contains(&addr) {
                0
            } else if self.prefetch_addr == addr {
                self.prefetch_addr = addr + 2;
                default_stall_time - self.prefetch_cycles_spent
            } else {
                self.prefetch_addr = addr + 2;
                default_stall_time
            }
        } else { default_stall_time };
        1 +
        if access_len == 2 { self.get_rom_access_time(wait_state, Cycle::S, 1, addr + 2) } else { 0 } +
        stall_time
    }

    pub fn clock_prefetch(&mut self, cycles: u32) {
        if self.use_prefetch && self.can_prefetch {
            for _ in 0..cycles {
                let prefetch_time = WaitStateControl::S_ACCESS_TIMINGS[self.prefetch_waitstate]
                    [self.s_wait_state_settings[self.prefetch_waitstate]];
                if self.prefetch_cycles_spent >= prefetch_time {
                    if self.prefetch.len() == 8 { self.prefetch.pop_front(); }
                    assert!(self.prefetch.len() < 8);
                    self.prefetch.push_back(self.prefetch_addr);
                    self.prefetch_addr += 2;
                    self.prefetch_cycles_spent = 0;
                } else { self.prefetch_cycles_spent += 1 }
            }
        }
        self.can_prefetch = true;
    }

    pub fn get_sram_access_time(&self, cycle_type: Cycle) -> u32 {
        assert_ne!(cycle_type, Cycle::I);
        1 + WaitStateControl::SRAM_ACCESS_TIMINGS[self.sram_setting]
    }
}

impl IORegister for WaitStateControl {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.s_wait_state_settings[1] << 7 | self.n_wait_state_settings[1] << 5 |
                    self.s_wait_state_settings[0] << 4 | self.n_wait_state_settings[0] << 2 | self.sram_setting) as u8,
            1 => ((self.type_flag as usize) << 7 | (self.use_prefetch as usize) << 6 | self.phi_terminal_out << 3 |
                self.s_wait_state_settings[2] << 2 | self.n_wait_state_settings[2]) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, _scheduler: &mut Scheduler, byte: u8, value: u8) {
        match byte {
            0 => {
                let value = value as usize;
                self.sram_setting = value & 0x3;
                self.n_wait_state_settings[0] = (value >> 2) & 0x3;
                self.s_wait_state_settings[0] = (value >> 4) & 0x1;
                self.n_wait_state_settings[1] = (value >> 5) & 0x3;
                self.s_wait_state_settings[1] = (value >> 7) & 0x1;
            },
            1 => {
                let value = value as usize;
                self.n_wait_state_settings[2] = (value >> 0) & 0x3;
                self.s_wait_state_settings[2] = (value >> 2) & 0x1;
                self.phi_terminal_out = (value >> 3) & 0x3;
                self.use_prefetch = (value >> 6) & 0x1 != 0;
                // Type Flag is read only
            }
            _ => unreachable!(),
        }
    }
}



mod mgba_test_suite {
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

    pub struct MGBATestSuite {
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
            self.enable == 0xC0DE
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
                        let null_byte_pos = self.buffer.iter().position(|&c| c == '\0')
                            .unwrap_or(self.buffer.len());
                        let message: String = self.buffer.iter().take(null_byte_pos).collect();
                        
                        if message.contains("PASS") { return }
                        let show_info = message.contains("FAIL") &&
                            !message.split(' ').skip(1).take(1).collect::<String>().contains("P");
                        let show_debug = !message.split(' ').rev().take(1).collect::<String>().contains("P");
                        match MGBALogLevel::new(self.flags & 0x7) {
                            Fatal => error!("{}", message),
                            Error => error!("{}", message),
                            Warn => warn!("{}", message),
                            Info => if show_info { info!("{}", message) },
                            Debug => if show_debug { debug!("{}", message) },
                        }
                    }
                },
                _ => (), 
            }
        }
    }
}
