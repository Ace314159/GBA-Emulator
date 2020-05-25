mod registers;

use super::MemoryHandler;
use super::IORegister;

use registers::*;

pub struct DMA {
    channel0: DMAChannel,
    channel1: DMAChannel,
    channel2: DMAChannel,
    channel3: DMAChannel,
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            channel0: DMAChannel::new(false, false, false),
            channel1: DMAChannel::new(true, true, false),
            channel2: DMAChannel::new(true, false, false),
            channel3: DMAChannel::new(true, true, true),
        }
    }
}

impl MemoryHandler for DMA {
    fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x040000B0 => self.channel0.sad.read(0),
            0x040000B1 => self.channel0.sad.read(1),
            0x040000B2 => self.channel0.sad.read(2),
            0x040000B3 => self.channel0.sad.read(3),
            0x040000B4 => self.channel0.dad.read(0),
            0x040000B5 => self.channel0.dad.read(1),
            0x040000B6 => self.channel0.dad.read(2),
            0x040000B7 => self.channel0.dad.read(3),
            0x040000B8 => self.channel0.cnt_l.read(0),
            0x040000B9 => self.channel0.cnt_l.read(1),
            0x040000BA => self.channel0.cnt_h.read(0),
            0x040000BB => self.channel0.cnt_h.read(1),
            0x040000BC => self.channel1.sad.read(0),
            0x040000BD => self.channel1.sad.read(1),
            0x040000BE => self.channel1.sad.read(2),
            0x040000BF => self.channel1.sad.read(3),
            0x040000C0 => self.channel1.dad.read(0),
            0x040000C1 => self.channel1.dad.read(1),
            0x040000C2 => self.channel1.dad.read(2),
            0x040000C3 => self.channel1.dad.read(3),
            0x040000C4 => self.channel1.cnt_l.read(0),
            0x040000C5 => self.channel1.cnt_l.read(1),
            0x040000C6 => self.channel1.cnt_h.read(0),
            0x040000C7 => self.channel1.cnt_h.read(1),
            0x040000C8 => self.channel2.sad.read(0),
            0x040000C9 => self.channel2.sad.read(1),
            0x040000CA => self.channel2.sad.read(2),
            0x040000CB => self.channel2.sad.read(3),
            0x040000CC => self.channel2.dad.read(0),
            0x040000CD => self.channel2.dad.read(1),
            0x040000CE => self.channel2.dad.read(2),
            0x040000CF => self.channel2.dad.read(3),
            0x040000D0 => self.channel2.cnt_l.read(0),
            0x040000D1 => self.channel2.cnt_l.read(1),
            0x040000D2 => self.channel2.cnt_h.read(0),
            0x040000D3 => self.channel2.cnt_h.read(1),
            0x040000D4 => self.channel3.sad.read(0),
            0x040000D5 => self.channel3.sad.read(1),
            0x040000D6 => self.channel3.sad.read(2),
            0x040000D7 => self.channel3.sad.read(3),
            0x040000D8 => self.channel3.dad.read(0),
            0x040000D9 => self.channel3.dad.read(1),
            0x040000DA => self.channel3.dad.read(2),
            0x040000DB => self.channel3.dad.read(3),
            0x040000DC => self.channel3.cnt_l.read(0),
            0x040000DD => self.channel3.cnt_l.read(1),
            0x040000DE => self.channel3.cnt_h.read(0),
            0x040000DF => self.channel3.cnt_h.read(1),
            _ => panic!("Reading from Invalid DMA Address {}", addr),
        }
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x040000B0 => self.channel0.sad.write(0, value),
            0x040000B1 => self.channel0.sad.write(1, value),
            0x040000B2 => self.channel0.sad.write(2, value),
            0x040000B3 => self.channel0.sad.write(3, value),
            0x040000B4 => self.channel0.dad.write(0, value),
            0x040000B5 => self.channel0.dad.write(1, value),
            0x040000B6 => self.channel0.dad.write(2, value),
            0x040000B7 => self.channel0.dad.write(3, value),
            0x040000B8 => self.channel0.cnt_l.write(0, value),
            0x040000B9 => self.channel0.cnt_l.write(1, value),
            0x040000BA => self.channel0.cnt_h.write(0, value),
            0x040000BB => self.channel0.cnt_h.write(1, value),
            0x040000BC => self.channel1.sad.write(0, value),
            0x040000BD => self.channel1.sad.write(1, value),
            0x040000BE => self.channel1.sad.write(2, value),
            0x040000BF => self.channel1.sad.write(3, value),
            0x040000C0 => self.channel1.dad.write(0, value),
            0x040000C1 => self.channel1.dad.write(1, value),
            0x040000C2 => self.channel1.dad.write(2, value),
            0x040000C3 => self.channel1.dad.write(3, value),
            0x040000C4 => self.channel1.cnt_l.write(0, value),
            0x040000C5 => self.channel1.cnt_l.write(1, value),
            0x040000C6 => self.channel1.cnt_h.write(0, value),
            0x040000C7 => self.channel1.cnt_h.write(1, value),
            0x040000C8 => self.channel2.sad.write(0, value),
            0x040000C9 => self.channel2.sad.write(1, value),
            0x040000CA => self.channel2.sad.write(2, value),
            0x040000CB => self.channel2.sad.write(3, value),
            0x040000CC => self.channel2.dad.write(0, value),
            0x040000CD => self.channel2.dad.write(1, value),
            0x040000CE => self.channel2.dad.write(2, value),
            0x040000CF => self.channel2.dad.write(3, value),
            0x040000D0 => self.channel2.cnt_l.write(0, value),
            0x040000D1 => self.channel2.cnt_l.write(1, value),
            0x040000D2 => self.channel2.cnt_h.write(0, value),
            0x040000D3 => self.channel2.cnt_h.write(1, value),
            0x040000D4 => self.channel3.sad.write(0, value),
            0x040000D5 => self.channel3.sad.write(1, value),
            0x040000D6 => self.channel3.sad.write(2, value),
            0x040000D7 => self.channel3.sad.write(3, value),
            0x040000D8 => self.channel3.dad.write(0, value),
            0x040000D9 => self.channel3.dad.write(1, value),
            0x040000DA => self.channel3.dad.write(2, value),
            0x040000DB => self.channel3.dad.write(3, value),
            0x040000DC => self.channel3.cnt_l.write(0, value),
            0x040000DD => self.channel3.cnt_l.write(1, value),
            0x040000DE => self.channel3.cnt_h.write(0, value),
            0x040000DF => self.channel3.cnt_h.write(1, value),
            _ => panic!("Writing to Invalid DMA Address {}", addr),
        }
    }
}

struct DMAChannel {
    pub sad: Address,
    pub dad: Address,
    pub cnt_l: WordCount,
    pub cnt_h: DMACNT,
}

impl DMAChannel {
    pub fn new(src_any_memory: bool, dest_any_memory: bool, count_is16bit: bool) -> DMAChannel {
        DMAChannel {
            sad: Address::new(src_any_memory),
            dad: Address::new(dest_any_memory),
            cnt_l: WordCount::new(count_is16bit),
            cnt_h: DMACNT::new(count_is16bit),
        }
    }
}
