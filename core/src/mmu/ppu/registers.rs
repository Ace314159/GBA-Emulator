use super::IORegister;

#[derive(Clone, Copy)]
pub enum BGMode {
    Mode0 = 0,
    Mode1 = 1,
    Mode2 = 2,
    Mode3 = 3,
    Mode4 = 4,
    Mode5 = 5,
}

impl BGMode {
    pub fn get(mode: u16) -> BGMode {
        use BGMode::*;
        match mode {
            0 => Mode0,
            1 => Mode1,
            2 => Mode2,
            3 => Mode3,
            4 => Mode4,
            5 => Mode5,
            _ => panic!("Invalid BG Mode!"),
        }
    }
}

bitflags! {
    pub struct DISPCNTFlags: u16 {
        const CGB_MODE = 1 << 3;
        const DISPLAY_FRAME_SELECT = 1 << 4;
        const HBLANK_INTERVAL_FREE = 1 << 5;
        const OBJ_CHAR_MAPPING = 1 << 6;
        const FORCED_BLANK = 1 << 7;
        const DISPLAY_BG0 = 1 << 8;
        const DISPLAY_BG1 = 1 << 9;
        const DISPLAY_BG2 = 1 << 10;
        const DISPLAY_BG3 = 1 << 11;
        const DISPLAY_OBJ = 1 << 12;
        const DISPLAY_WINDOW0 = 1 << 13;
        const DISPLAY_WINDOW1 = 1 << 14;
        const DISPLAY_OBJ_WINDOW = 1 << 15;
    }
}

pub struct DISPCNT {
    pub flags: DISPCNTFlags,
    pub mode: BGMode,
}

impl DISPCNT {
    pub fn new() -> DISPCNT {
        DISPCNT {
            flags: DISPCNTFlags::empty(),
            mode: BGMode::Mode0,
        }
    }
}

impl IORegister for DISPCNT {
    fn read(&self) -> u16 {
        self.flags.bits | (self.mode as u16)
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask;
        self.flags.bits = value & DISPCNTFlags::all().bits();
        self.mode = BGMode::get(value & 0x7);
    }
}

bitflags! {
    pub struct DISPSTATFlags: u16 {
        const VBLANK = 1 << 0;
        const HBLANK = 1 << 1;
        const VCOUNTER = 1 << 2;
        const VBLANK_IRQ_ENABLE = 1 << 3;
        const HBLANK_IRQ_ENABLE = 1 << 4;
        const VCOUNTER_IRQ_ENALBE = 1 << 5;
    }
}

pub struct DISPSTAT {
    pub flags: DISPSTATFlags,
    pub vcount_setting: u8,
}

impl DISPSTAT {
    pub fn new() -> DISPSTAT {
        DISPSTAT {
            flags: DISPSTATFlags::empty(),
            vcount_setting: 0,
        }
    }
}

impl IORegister for DISPSTAT {
    fn read(&self) -> u16 {
        self.flags.bits | (self.vcount_setting as u16)
    }

    fn write(&mut self, mask: u16, value: u16) {
        let value = value & mask;
        self.flags.bits = value & DISPCNTFlags::all().bits();
        self.vcount_setting = value as u8;
    }
}
