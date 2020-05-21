mod registers;

use registers::*;

pub struct DMA {
    pub sad: Address,
    pub dad: Address,
    pub cnt_l: WordCount,
}

impl DMA {
    pub fn new(src_any_memory: bool, dest_any_memory: bool, count_is16bit: bool) -> DMA {
        DMA {
            sad: Address::new(src_any_memory),
            dad: Address::new(dest_any_memory),
            cnt_l: WordCount::new(count_is16bit),
        }
    }
}
