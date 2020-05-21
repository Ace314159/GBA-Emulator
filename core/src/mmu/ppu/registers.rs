use std::ops::Deref;

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
    pub fn get(mode: u8) -> BGMode {
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
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.flags.bits as u8) | (self.mode as u8),
            1 => (self.flags.bits >> 8) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.mode = BGMode::get(value & 0x7);
                self.flags.bits = self.flags.bits & !0x00FF | (value as u16) & DISPSTATFlags::all().bits; 
            },
            1 => self.flags.bits = self.flags.bits & !0xFF00 | (value as u16) << 8 & DISPSTATFlags::all().bits,
            _ => panic!("Invalid Byte!"),
        }
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
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.flags.bits as u8,
            1 => self.vcount_setting as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.flags.bits = (value as u16) & DISPSTATFlags::all().bits,
            1 => self.vcount_setting = value as u8,
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct BG01CNT {
    priority: u8,
    tile_block: u8,
    mosaic: bool,
    use_palettes: bool,
    map_block: u8,
    screen_size: u8,
}


impl BG01CNT {
    pub fn new() -> BG01CNT {
        BG01CNT {
            priority: 0,
            tile_block: 0,
            mosaic: false,
            use_palettes: false,
            map_block: 0,
            screen_size: 0, 
        }
    }
}

impl IORegister for BG01CNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.use_palettes as u8) << 7 | (self.mosaic as u8) << 6 | self.tile_block << 2 | self.priority,
            1 => self.screen_size << 6 | self.map_block,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.priority = value & 0x3;
                self.tile_block = value >> 2 & 0x3;
                self.mosaic = value >> 6 & 0x1 != 0;
                self.use_palettes = value >> 7 & 0x1 != 0;
            },
            1 => {
                self.map_block = value & 0x3;
                self.screen_size = value >> 6 & 0x3;
            },
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct BG23CNT {
    bg01cnt: BG01CNT,
    wrap: bool,
}

impl BG23CNT {
    pub fn new() -> BG23CNT {
        BG23CNT {
            bg01cnt: BG01CNT::new(),
            wrap: false,
        }
    }
}

impl Deref for BG23CNT {
    type Target = BG01CNT;
    fn deref(&self) -> &BG01CNT {
        &self.bg01cnt
    }
}

impl IORegister for BG23CNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.bg01cnt.read(0),
            1 => self.bg01cnt.read(1) | (self.wrap as u8) << 5,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.bg01cnt.write(1, value),
            1 => {
                self.bg01cnt.write(0, value);
                self.wrap = value >> 5 & 0x1 != 0;
            },
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct OFS {
    offset: u16,
}

impl OFS {
    pub fn new() -> OFS {
        OFS {
            offset: 0,
        }
    }
}

impl IORegister for OFS {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.offset as u8,
            1 => (self.offset >> 8) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }
    
    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.offset = self.offset & !0xFF | value as u16,
            1 => self.offset = self.offset & !0x100 | (value as u16) << 8,
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct RotationScalingParameter {
    fractional: u8,
    integer: u8,
    sign: bool,
}

impl RotationScalingParameter {
    pub fn new() -> RotationScalingParameter {
        RotationScalingParameter {
            fractional: 0,
            integer: 0,
            sign: false,
        }
    }
}

impl IORegister for RotationScalingParameter {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.fractional,
            1 => (self.sign as u8) << 7 | self.integer,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.fractional = value,
            1 => {
                self.integer = value & 0x7F;
                self.sign = value >> 7 & 0x1 != 0;
            },
            _ => panic!("Invalid Byte!"),
        }
    }
}

pub struct ReferencePointCoord {
    fractional: u8,
    integer: u32,
    sign: bool,
}

impl ReferencePointCoord {
    pub fn new() -> ReferencePointCoord {
        ReferencePointCoord {
            fractional: 0,
            integer: 0,
            sign: false,
        }
    }
}

impl IORegister for ReferencePointCoord {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.fractional,
            1 => self.integer as u8,
            2 => (self.integer >> 8) as u8,
            3 => (self.sign as u8) << 3 | (self.integer >> 16) as u8,
            _ => panic!("Invalid Byte!"),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.fractional = self.fractional & !0xFF | value,
            1 => self.integer = self.integer & !0xFF | value as u32,
            2 => self.integer = self.integer & !0xFF00 | (value as u32) << 8,
            3 => {
                self.integer = self.integer & !0x70000 | ((value as u32) & 0x7) << 16;
                self.sign = value >> 4 & 0x1 != 0;
            },
            _ => panic!("Invalid Byte!"),
        }
    }
}
