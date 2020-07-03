use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use flume::{Receiver, Sender};

use crate::cpu::CPU;
use crate::io::IO;
pub use crate::io::{
    DebugSpecification, DebugWindows,
    keypad::KEYINPUT,
};

pub struct GBA {
    cpu: CPU,
    io: IO,
    next_frame_cycle: usize,
}

impl GBA {
    pub fn new(rom_file: PathBuf, render_tx: Sender<DebugWindows>, keypad_rx: Receiver<(KEYINPUT, bool)>) ->
        (GBA, Arc<Mutex<Vec<u16>>>, Arc<Mutex<DebugSpecification>>) {
        let bios = std::fs::read("gba_bios.bin").unwrap();
        let (mut io, pixels, debug_windows_spec) =
            IO::new(bios, rom_file, render_tx, keypad_rx);
        (GBA {
            cpu: CPU::new(false, &mut io),
            io,
            next_frame_cycle: 0,
        }, pixels, debug_windows_spec)
    }

    pub fn emulate_frame(&mut self) {
        self.io.poll_keypad_updates();
        // TODO: This will overflow on 32-bit systems
        self.next_frame_cycle += CLOCKS_PER_FRAME;
        while self.io.get_cycle() < self.next_frame_cycle {
            self.io.run_dma();
            self.cpu.handle_irq(&mut self.io);
            self.cpu.emulate_instr(&mut self.io);
        }
    }

    pub fn peek_mem(&self, region: VisibleMemoryRegion, addr: usize) -> u8 {
        self.io.peek_mem(region, addr as u32)
    }
}

pub const WIDTH: usize = 240;
pub const HEIGHT: usize = 160;
pub const SCALE: usize = 2;

pub const AUDIO_SAMPLE_RATE: usize = 0x8000;
pub const AUDIO_BUFFER_LEN: usize = 4096;
pub const CLOCK_FREQ: usize = 1 << 24;
pub const CLOCKS_PER_FRAME: usize = 280896;
pub const FRAME_PERIOD: std::time::Duration = std::time::Duration::from_nanos(
    1e9 as u64 * CLOCKS_PER_FRAME as u64 / CLOCK_FREQ as u64
);

#[derive(Clone, Copy)]
pub enum VisibleMemoryRegion {
    BIOS = 0,
    EWRAM = 1,
    IWRAM = 2,
    IO = 3,
    Palette = 4,
    VRAM = 5,
    OAM = 6,
    PakROM = 7,
    CartRAM = 8,
}

impl VisibleMemoryRegion {
    pub fn from_index(index: usize) -> Self {
        use VisibleMemoryRegion::*;
        match index {
            0 => BIOS,
            1 => EWRAM,
            2 => IWRAM,
            3 => IO,
            4 => Palette,
            5 => VRAM,
            6 => OAM,
            7 => PakROM,
            8 => CartRAM,
            _ => unreachable!(),
        }
    }

    pub fn get_name(&self) -> String {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => "BIOS",
            EWRAM => "EWRAM",
            IWRAM => "IWRAM",
            IO => "IO",
            Palette => "Palette",
            VRAM => "VRAM",
            OAM => "OAM",
            PakROM => "Pak ROM",
            CartRAM => "Cart RAM",
        }.to_string()
    }

    pub fn get_size(&self) -> usize {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => 0x4000,
            EWRAM => 0x4_0000,
            IWRAM => 0x8000,
            IO => 0x400,
            Palette => 0x400,
            VRAM => 0x1_8000,
            OAM => 0x400,
            PakROM => 0x0600_0000,
            CartRAM => 0x1_0000,
        }
    }

    pub fn get_start_addr(&self) -> u32 {
        use VisibleMemoryRegion::*;
        match *self {
            BIOS => 0x0000_0000,
            EWRAM => 0x0200_0000,
            IWRAM => 0x0300_0000,
            IO => 0x0400_0000,
            Palette => 0x0500_0000,
            VRAM => 0x0600_0000,
            OAM => 0x0700_0000,
            PakROM => 0x0800_0000,
            CartRAM => 0x0E00_0000,
        }
    }
}
