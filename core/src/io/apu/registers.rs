use super::IORegister;

pub struct SoundEnableFlags {
    channel1: bool,
    channel2: bool,
    channel3: bool,
    channel4: bool,
}

impl SoundEnableFlags {
    pub fn new() -> SoundEnableFlags {
        SoundEnableFlags {
            channel4: false,
            channel3: false,
            channel2: false,
            channel1: false,
        }
    }

    pub fn read(&self) -> u8 {
        (self.channel4 as u8) << 3 | (self.channel3 as u8) << 2 | (self.channel2 as u8) << 1 | (self.channel1 as u8) << 0
    }

    pub fn write(&mut self, value: u8) {
        self.channel1 = value >> 0 & 0x1 != 0;
        self.channel2 = value >> 1 & 0x1 != 0;
        self.channel3 = value >> 2 & 0x1 != 0;
        self.channel4 = value >> 3 & 0x1 != 0;
    }
}

pub struct DMASoundControl {
    enable_right: bool,
    enable_left: bool,
    timer_select: u8,
    reset_fifo: bool,
}

impl DMASoundControl {
    pub fn new() -> DMASoundControl {
        DMASoundControl {
            enable_right: false,
            enable_left: false,
            timer_select: 0,
            reset_fifo: false,
        }
    }

    pub fn read(&self) -> u8 {
        self.timer_select << 2 | (self.enable_left as u8) << 1 | (self.enable_right as u8) << 0
    }

    pub fn write(&mut self, value: u8) {
        self.enable_right = value >> 0 & 0x1 != 0;
        self.enable_left = value >> 1 & 0x1 != 0;
        self.timer_select = value >> 2 & 0x1;
        self.reset_fifo = value >> 3 & 0x1 != 0;
    }
}

pub struct SOUNDCNT {
    psg_master_volume_r: u8,
    psg_master_volume_l: u8,
    psg_enable_r: SoundEnableFlags,
    psg_enable_l: SoundEnableFlags,
    psg_volume: u8,
    dma_sound_a_vol: bool,
    dma_sound_b_vol: bool,
    dma_sound_a_cnt: DMASoundControl,
    dma_sound_b_cnt: DMASoundControl,
}

impl SOUNDCNT {
    pub fn new() -> SOUNDCNT {
        SOUNDCNT {
            psg_master_volume_r: 0,
            psg_master_volume_l: 0,
            psg_enable_r: SoundEnableFlags::new(),
            psg_enable_l: SoundEnableFlags::new(),
            psg_volume: 0,
            dma_sound_a_vol: false,
            dma_sound_b_vol: false,
            dma_sound_a_cnt: DMASoundControl::new(),
            dma_sound_b_cnt: DMASoundControl::new(),
        }
    }
}

impl IORegister for SOUNDCNT {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.psg_master_volume_l << 4 | self.psg_master_volume_r,
            1 => self.psg_enable_l.read() << 4 | self.psg_enable_r.read(),
            2 => (self.dma_sound_b_vol as u8) << 3 | (self.dma_sound_a_vol as u8) << 2 | (self.psg_volume),
            3 => self.dma_sound_b_cnt.read() << 4 | self.dma_sound_a_cnt.read(),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => {
                self.psg_master_volume_r = value & 0x7;
                self.psg_master_volume_l = value >> 4 & 0x7;
            },
            1 => {
                self.psg_enable_r.write(value & 0xF);
                self.psg_enable_l.write(value >> 4);
            },
            2 => {
                self.psg_volume = value & 0x3;
                self.dma_sound_a_vol = value >> 2 & 0x1 != 0;
                self.dma_sound_b_vol = value >> 3 & 0x1 != 0;
            },
            3 => {
                self.dma_sound_b_cnt.write(value & 0xF);
                self.dma_sound_a_cnt.write(value >> 4);
            },
            _ => unreachable!(),
        }
    }
}

pub struct SOUNDBIAS {
    bias_level: u16,
    amplitude_res: u8,
}

impl SOUNDBIAS {
    pub fn new() -> SOUNDBIAS {
        SOUNDBIAS {
            bias_level: 0x100,
            amplitude_res: 0,
        }
    }
}

impl IORegister for SOUNDBIAS {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.bias_level << 1) as u8,
            1 => self.amplitude_res << 6 | (self.bias_level >> 8) as u8,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.bias_level = self.bias_level & !0xFF | (value as u16) >> 1,
            1 => {
                self.bias_level = self.bias_level & !0x100 | (value as u16) << 8;
                self.amplitude_res = (value >> 6) & 0x3;
            },
            _ => unreachable!(),
        }
    }
}
