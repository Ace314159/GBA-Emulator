mod audio;
mod registers;
mod channel;

use super::IORegister;
use crate::gba;

use audio::Audio;
use registers::*;
use channel::Timer;
use channel::*;

pub struct APU {
    // Channels
    tone1: Tone,
    tone2: Tone,
    wave: Wave,
    // Sound Control Registers
    cnt: SOUNDCNT,
    bias: SOUNDBIAS,
    master_enable: bool,
    
    // Sound Generation
    audio: Audio,
    sequencer_step: u8,
    sequencer_clock: Timer<u16>,
    sample_clock: usize,
}

impl APU {
    const CLOCKS_PER_SAMPLE: usize = gba::CLOCK_FREQ / gba::AUDIO_SAMPLE_RATE;

    pub fn new() -> APU {
        APU {
            // Channels
            tone1: Tone::new(),
            tone2: Tone::new(),
            wave: Wave::new(),
            // Sound Control Registers
            cnt: SOUNDCNT::new(),
            bias: SOUNDBIAS::new(),
            master_enable: false,

            // Sound Generation
            audio: Audio::new(),
            sequencer_step: 0,
            sequencer_clock: Timer::new((gba::CLOCK_FREQ / 512) as u16),
            sample_clock: APU::CLOCKS_PER_SAMPLE,
        }
    }

    pub fn clock(&mut self) {
        if !self.master_enable { return }
        self.tone1.clock();
        self.tone2.clock();
        self.wave.clock();

        self.clock_sequencer();

        self.generate_sample();
    }

    pub fn clock_sequencer(&mut self) {
        if self.sequencer_clock.clock() {
            match self.sequencer_step {
                0 => self.clock_length_counters(),
                2 => { self.clock_length_counters(); self.tone1.sweep.clock() },
                4 => self.clock_length_counters(),
                6 => { self.clock_length_counters(); self.tone1.sweep.clock() },
                7 => self.clock_envelopes(),
                _ => assert!(self.sequencer_step < 8),
            }
            self.sequencer_step = (self.sequencer_step + 1) % 8;
        }
    }

    fn clock_length_counters(&mut self) {
        self.tone1.length_counter.clock();
        self.tone2.length_counter.clock();
        self.wave.length_counter.clock();
    }

    fn clock_envelopes(&mut self) {
        self.tone1.envelope.clock();
        self.tone2.envelope.clock();
    }

    fn generate_sample(&mut self) {
        self.sample_clock -= 1;
        if self.sample_clock == 0 {
            let channel1_sample = self.tone1.generate_sample();
            let channel2_sample = self.tone2.generate_sample();
            let channel3_sample = self.wave.generate_sample();
            let mut left_sample = 0;
            let mut right_sample = 0;

            left_sample += self.cnt.psg_enable_l.channel1 as i16 * channel1_sample;
            left_sample += self.cnt.psg_enable_l.channel2 as i16 * channel2_sample;
            left_sample += self.cnt.psg_enable_l.channel3 as i16 * channel3_sample;
            right_sample += self.cnt.psg_enable_r.channel1 as i16 * channel1_sample;
            right_sample += self.cnt.psg_enable_r.channel2 as i16 * channel2_sample;
            right_sample += self.cnt.psg_enable_r.channel3 as i16 * channel3_sample;

            left_sample *= self.cnt.psg_master_volume_l as i16;
            right_sample *= self.cnt.psg_master_volume_r as i16;

            self.audio.queue(left_sample, right_sample);
            self.sample_clock = APU::CLOCKS_PER_SAMPLE;
        }
    }
}

impl APU {
    pub fn read_register(&self, addr: u32) -> u8 {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x060 => self.tone1.read(0),
            0x061 => self.tone1.read(1),
            0x062 => self.tone1.read(2),
            0x063 => self.tone1.read(3),
            0x064 => self.tone1.read(4),
            0x065 => self.tone1.read(5),
            0x066 => self.tone1.read(6),
            0x067 => self.tone1.read(7),
            0x068 => self.tone2.read(0 + 2),
            0x069 => self.tone2.read(1 + 2),
            0x06A => 0,
            0x06B => 0,
            0x06C => self.tone2.read(4),
            0x06D => self.tone2.read(5),
            0x06E => self.tone2.read(6),
            0x06F => self.tone2.read(7),
            0x070 => self.wave.read(0),
            0x071 => self.wave.read(1),
            0x072 => self.wave.read(2),
            0x073 => self.wave.read(3),
            0x074 => self.wave.read(4),
            0x075 => self.wave.read(5),
            0x076 => self.wave.read(6),
            0x077 => self.wave.read(7),
            0x080 => self.cnt.read(0),
            0x081 => self.cnt.read(1),
            0x082 => self.cnt.read(2),
            0x083 => self.cnt.read(3),
            0x084 => (self.master_enable as u8) << 7 | (self.wave.is_on() as u8) << 3 | (self.tone2.is_on() as u8) << 1 |
                        (self.tone1.is_on() as u8) << 0,
            0x085 ..= 0x087 => 0,
            0x088 => self.bias.read(0),
            0x089 => self.bias.read(1),
            0x08A ..= 0x08F => 0,
            0x090 ..= 0x09F => self.wave.read_wave_ram(addr - 0x04000090),
            _ => { warn!("Ignoring APU Read at 0x{:08X}", addr); 0 },
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x060 => self.tone1.write(0, value),
            0x061 => self.tone1.write(1, value),
            0x062 => self.tone1.write(2, value),
            0x063 => self.tone1.write(3, value),
            0x064 => self.tone1.write(4, value),
            0x065 => self.tone1.write(5, value),
            0x066 => self.tone1.write(6, value),
            0x067 => self.tone1.write(7, value),
            0x068 => self.tone2.write(0 + 2, value),
            0x069 => self.tone2.write(1 + 2, value),
            0x06A => (),
            0x06B => (),
            0x06C => self.tone2.write(4, value),
            0x06D => self.tone2.write(5, value),
            0x06E => self.tone2.write(6, value),
            0x06F => self.tone2.write(7, value),
            0x070 => self.wave.write(0, value),
            0x071 => self.wave.write(1, value),
            0x072 => self.wave.write(2, value),
            0x073 => self.wave.write(3, value),
            0x074 => self.wave.write(4, value),
            0x075 => self.wave.write(5, value),
            0x076 => self.wave.write(6, value),
            0x077 => self.wave.write(7, value),
            0x080 => self.cnt.write(0, value),
            0x081 => self.cnt.write(1, value),
            0x082 => self.cnt.write(2, value),
            0x083 => self.cnt.write(3, value),
            0x084 => {
                let prev = self.master_enable;
                self.master_enable = value >> 7 & 0x1 != 0;
                if !prev && self.master_enable {
                    self.tone1 = Tone::new();
                    self.tone2 = Tone::new();
                    self.cnt.write(0, value);
                    self.cnt.write(1, value);
                }
            },
            0x085 ..= 0x087 => (),
            0x088 => self.bias.write(0, value),
            0x089 => self.bias.write(1, value),
            0x08A ..= 0x08F => (),
            0x090 ..= 0x09F => self.wave.write_wave_ram(addr - 0x04000090, value),
            _ => warn!("Ignoring APU Write 0x{:08X} = {:02X}", addr, value),
        }
    }
}
