mod audio;
mod registers;
mod channel;

use super::IORegister;
use crate::gba;

use audio::Audio;
use registers::*;
use channel::*;

pub struct APU {
    // Channels
    tone: Tone,
    // Sound Control Registers
    cnt: SOUNDCNT,
    bias: SOUNDBIAS,
    cnt_x: SOUNDCNTX,
    
    // Sound Generation
    audio: Audio,
    sequencer_step: u8,
    sequencer_clock: u16,
    sample_clock: usize,
}

impl APU {
    const CLOCKS_PER_SAMPLE: usize = gba::CLOCK_FREQ / gba::AUDIO_SAMPLE_RATE;

    pub fn new() -> APU {
        APU {
            // Channels
            tone: Tone::new(),
            // Sound Control Registers
            cnt: SOUNDCNT::new(),
            bias: SOUNDBIAS::new(),
            cnt_x: SOUNDCNTX::new(),

            // Sound Generation
            audio: Audio::new(),
            sequencer_step: 0,
            sequencer_clock: 0,
            sample_clock: APU::CLOCKS_PER_SAMPLE,
        }
    }

    pub fn clock(&mut self) {
        self.tone.clock();

        self.clock_sequencer();

        self.generate_sample();
    }

    pub fn clock_sequencer(&mut self) {
        if self.sequencer_clock == 0 {
            match self.sequencer_step {
                0 ..= 6 => (),
                7 => self.tone.envelope.clock(),
                _ => unreachable!(),
            }
            self.sequencer_step = (self.sequencer_step + 1) % 8;
            self.sequencer_clock = 0x8000;
        } else { self.sequencer_clock -= 1 }
    }

    fn generate_sample(&mut self) {
        self.sample_clock -= 1;
        if self.sample_clock == 0 {
            let sample = self.tone.generate_sample();
            self.audio.queue(sample, sample);
            self.sample_clock = APU::CLOCKS_PER_SAMPLE;
        }
    }
}

impl APU {
    pub fn read_register(&self, addr: u32) -> u8 {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x068 => self.tone.read(0),
            0x069 => self.tone.read(1),
            0x06A => self.tone.read(2),
            0x06B => self.tone.read(3),
            0x06C => self.tone.read(4),
            0x06D => self.tone.read(5),
            0x06E => self.tone.read(6),
            0x06F => self.tone.read(7),
            0x080 => self.cnt.read(0),
            0x081 => self.cnt.read(1),
            0x082 => self.cnt.read(2),
            0x083 => self.cnt.read(3),
            0x084 => self.cnt_x.read(0),
            0x085 => self.cnt_x.read(1),
            0x086 => self.cnt_x.read(2),
            0x087 => self.cnt_x.read(3),
            0x088 => self.bias.read(0),
            0x089 => self.bias.read(1),
            0x08A ..= 0x08F => 0,
            _ => { warn!("Ignoring APU Read at 0x{:08X}", addr); 0 },
        }
    }

    pub fn write_register(&mut self, addr: u32, value: u8) {
        assert_eq!(addr >> 12, 0x04000);
        match addr & 0xFFF {
            0x068 => self.tone.write(0, value),
            0x069 => self.tone.write(1, value),
            0x06A => self.tone.write(2, value),
            0x06B => self.tone.write(3, value),
            0x06C => self.tone.write(4, value),
            0x06D => self.tone.write(5, value),
            0x06E => self.tone.write(6, value),
            0x06F => self.tone.write(7, value),
            0x080 => self.cnt.write(0, value),
            0x081 => self.cnt.write(1, value),
            0x082 => self.cnt.write(2, value),
            0x083 => self.cnt.write(3, value),
            0x084 => self.cnt_x.write(0, value),
            0x085 => self.cnt_x.write(1, value),
            0x086 => self.cnt_x.write(1, value),
            0x087 => self.cnt_x.write(1, value),
            0x088 => self.bias.write(0, value),
            0x089 => self.bias.write(1, value),
            0x08A ..= 0x08F => (),
            _ => warn!("Ignoring APU Write 0x{:08X} = {:02X}", addr, value),
        }
    }
}
