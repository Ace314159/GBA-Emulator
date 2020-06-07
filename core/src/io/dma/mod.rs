mod registers;

use super::IORegister;

use registers::*;

pub struct DMA {
    pub channels: [DMAChannel; 4],
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            channels: [
                DMAChannel::new(false, false, false),
                DMAChannel::new(true, true, false),
                DMAChannel::new(true, false, false),
                DMAChannel::new(true, true, true),
            ],
        }
    }

    pub fn get_channel_running(&mut self, hblank_called: bool, vblank_called: bool) -> usize {
        for (i, channel) in self.channels.iter().enumerate() {
            if (*channel).needs_to_transfer(hblank_called, vblank_called) { return i }
        }
        return 4;
    }
}

pub struct DMAChannel {
    pub sad_latch: u32,
    pub dad_latch: u32,
    pub count_latch: u32,

    sad: Address,
    dad: Address,
    pub count: WordCount,
    pub cnt: DMACNT,
}

impl DMAChannel {
    pub fn new(src_any_memory: bool, dest_any_memory: bool, count_is16bit: bool) -> DMAChannel {
        DMAChannel {
            sad_latch: 0,
            dad_latch: 0,
            count_latch: 0,

            sad: Address::new(src_any_memory),
            dad: Address::new(dest_any_memory),
            count: WordCount::new(count_is16bit),
            cnt: DMACNT::new(count_is16bit),
        }
    }

    pub fn needs_to_transfer(&self, hblank_called: bool, vblank_called: bool) -> bool {
        if !self.cnt.enable { return false }
        match self.cnt.start_timing {
            0 => true,
            1 => vblank_called,
            2 => hblank_called,
            3 => { warn!("Special DMA not implemented!"); false }, // TODO: Special
            _ => unreachable!(),
        }
    }

    pub fn latch(&mut self) {
        self.sad_latch = self.sad.addr;
        self.dad_latch = self.dad.addr;
        self.count_latch = if self.count.count == 0 { self.count.get_max() + 1 } else { self.count.count as u32 };
    }
}

impl IORegister for DMAChannel {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0x0 => self.sad.read(0),
            0x1 => self.sad.read(1),
            0x2 => self.sad.read(2),
            0x3 => self.sad.read(3),
            0x4 => self.dad.read(0),
            0x5 => self.dad.read(1),
            0x6 => self.dad.read(2),
            0x7 => self.dad.read(3),
            0x8 => self.count.read(0),
            0x9 => self.count.read(1),
            0xA => self.cnt.read(0),
            0xB => self.cnt.read(1),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0x0 => self.sad.write(0, value),
            0x1 => self.sad.write(1, value),
            0x2 => self.sad.write(2, value),
            0x3 => self.sad.write(3, value),
            0x4 => self.dad.write(0, value),
            0x5 => self.dad.write(1, value),
            0x6 => self.dad.write(2, value),
            0x7 => self.dad.write(3, value),
            0x8 => self.count.write(0, value),
            0x9 => self.count.write(1, value),
            0xA => self.cnt.write(0, value),
            0xB => {
                let prev_enable = self.cnt.enable;
                self.cnt.write(1, value);
                if !prev_enable && self.cnt.enable { self.latch() }
            },
            _ => unreachable!(),
        }
    }
}
