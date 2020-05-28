mod registers;

use super::MemoryHandler;
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

impl MemoryHandler for DMA {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x040000B0 => self.channels[0].sad.read(0),
            0x040000B1 => self.channels[0].sad.read(1),
            0x040000B2 => self.channels[0].sad.read(2),
            0x040000B3 => self.channels[0].sad.read(3),
            0x040000B4 => self.channels[0].dad.read(0),
            0x040000B5 => self.channels[0].dad.read(1),
            0x040000B6 => self.channels[0].dad.read(2),
            0x040000B7 => self.channels[0].dad.read(3),
            0x040000B8 => self.channels[0].count.read(0),
            0x040000B9 => self.channels[0].count.read(1),
            0x040000BA => self.channels[0].cnt.read(0),
            0x040000BB => self.channels[0].cnt.read(1),
            0x040000BC => self.channels[1].sad.read(0),
            0x040000BD => self.channels[1].sad.read(1),
            0x040000BE => self.channels[1].sad.read(2),
            0x040000BF => self.channels[1].sad.read(3),
            0x040000C0 => self.channels[1].dad.read(0),
            0x040000C1 => self.channels[1].dad.read(1),
            0x040000C2 => self.channels[1].dad.read(2),
            0x040000C3 => self.channels[1].dad.read(3),
            0x040000C4 => self.channels[1].count.read(0),
            0x040000C5 => self.channels[1].count.read(1),
            0x040000C6 => self.channels[1].cnt.read(0),
            0x040000C7 => self.channels[1].cnt.read(1),
            0x040000C8 => self.channels[2].sad.read(0),
            0x040000C9 => self.channels[2].sad.read(1),
            0x040000CA => self.channels[2].sad.read(2),
            0x040000CB => self.channels[2].sad.read(3),
            0x040000CC => self.channels[2].dad.read(0),
            0x040000CD => self.channels[2].dad.read(1),
            0x040000CE => self.channels[2].dad.read(2),
            0x040000CF => self.channels[2].dad.read(3),
            0x040000D0 => self.channels[2].count.read(0),
            0x040000D1 => self.channels[2].count.read(1),
            0x040000D2 => self.channels[2].cnt.read(0),
            0x040000D3 => self.channels[2].cnt.read(1),
            0x040000D4 => self.channels[3].sad.read(0),
            0x040000D5 => self.channels[3].sad.read(1),
            0x040000D6 => self.channels[3].sad.read(2),
            0x040000D7 => self.channels[3].sad.read(3),
            0x040000D8 => self.channels[3].dad.read(0),
            0x040000D9 => self.channels[3].dad.read(1),
            0x040000DA => self.channels[3].dad.read(2),
            0x040000DB => self.channels[3].dad.read(3),
            0x040000DC => self.channels[3].count.read(0),
            0x040000DD => self.channels[3].count.read(1),
            0x040000DE => self.channels[3].cnt.read(0),
            0x040000DF => self.channels[3].cnt.read(1),
            _ => panic!("Reading from Invalid DMA Address {}", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x040000B0 => self.channels[0].sad.write(0, value),
            0x040000B1 => self.channels[0].sad.write(1, value),
            0x040000B2 => self.channels[0].sad.write(2, value),
            0x040000B3 => self.channels[0].sad.write(3, value),
            0x040000B4 => self.channels[0].dad.write(0, value),
            0x040000B5 => self.channels[0].dad.write(1, value),
            0x040000B6 => self.channels[0].dad.write(2, value),
            0x040000B7 => self.channels[0].dad.write(3, value),
            0x040000B8 => self.channels[0].count.write(0, value),
            0x040000B9 => self.channels[0].count.write(1, value),
            0x040000BA => self.channels[0].cnt.write(0, value),
            0x040000BB => self.channels[0].cnt.write(1, value),
            0x040000BC => self.channels[1].sad.write(0, value),
            0x040000BD => self.channels[1].sad.write(1, value),
            0x040000BE => self.channels[1].sad.write(2, value),
            0x040000BF => self.channels[1].sad.write(3, value),
            0x040000C0 => self.channels[1].dad.write(0, value),
            0x040000C1 => self.channels[1].dad.write(1, value),
            0x040000C2 => self.channels[1].dad.write(2, value),
            0x040000C3 => self.channels[1].dad.write(3, value),
            0x040000C4 => self.channels[1].count.write(0, value),
            0x040000C5 => self.channels[1].count.write(1, value),
            0x040000C6 => self.channels[1].cnt.write(0, value),
            0x040000C7 => self.channels[1].cnt.write(1, value),
            0x040000C8 => self.channels[2].sad.write(0, value),
            0x040000C9 => self.channels[2].sad.write(1, value),
            0x040000CA => self.channels[2].sad.write(2, value),
            0x040000CB => self.channels[2].sad.write(3, value),
            0x040000CC => self.channels[2].dad.write(0, value),
            0x040000CD => self.channels[2].dad.write(1, value),
            0x040000CE => self.channels[2].dad.write(2, value),
            0x040000CF => self.channels[2].dad.write(3, value),
            0x040000D0 => self.channels[2].count.write(0, value),
            0x040000D1 => self.channels[2].count.write(1, value),
            0x040000D2 => self.channels[2].cnt.write(0, value),
            0x040000D3 => self.channels[2].cnt.write(1, value),
            0x040000D4 => self.channels[3].sad.write(0, value),
            0x040000D5 => self.channels[3].sad.write(1, value),
            0x040000D6 => self.channels[3].sad.write(2, value),
            0x040000D7 => self.channels[3].sad.write(3, value),
            0x040000D8 => self.channels[3].dad.write(0, value),
            0x040000D9 => self.channels[3].dad.write(1, value),
            0x040000DA => self.channels[3].dad.write(2, value),
            0x040000DB => self.channels[3].dad.write(3, value),
            0x040000DC => self.channels[3].count.write(0, value),
            0x040000DD => self.channels[3].count.write(1, value),
            0x040000DE => self.channels[3].cnt.write(0, value),
            0x040000DF => self.channels[3].cnt.write(1, value),
            _ => panic!("Writing to Invalid DMA Address {}", addr),
        }
    }
}

pub struct DMAChannel {
    pub sad: Address,
    pub dad: Address,
    pub count: WordCount,
    pub cnt: DMACNT,
}

impl DMAChannel {
    pub fn new(src_any_memory: bool, dest_any_memory: bool, count_is16bit: bool) -> DMAChannel {
        DMAChannel {
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
            1 => hblank_called,
            2 => vblank_called,
            3 => false, // TODO: Special
            _ => panic!("Invalid DMA Start Timing: {}", self.cnt.start_timing),
        }
    } 
}
