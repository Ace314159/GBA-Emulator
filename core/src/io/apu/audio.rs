extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
pub use sdl2::audio::AudioDevice;

use crate::gba;

pub struct Audio {
    buffer: [i16; gba::AUDIO_BUFFER_LEN],
    read_i: usize,
    write_i: usize,
    count: usize,
}

impl Audio {
    const DESIRED_SPEC: AudioSpecDesired = AudioSpecDesired {
        freq: Some(gba::AUDIO_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: None,
    };

    const VOLUME_FACTOR: i16 = 8;

    pub fn new() -> AudioDevice<Audio> {
        let sdl_ctx = sdl2::init().unwrap();
        let audio_subsystem = sdl_ctx.audio().unwrap();

        let device = audio_subsystem
        .open_playback(None, &Audio::DESIRED_SPEC, |_spec| {
            Audio {
                buffer: [0; gba::AUDIO_BUFFER_LEN],
                read_i: 0,
                write_i: 0,
                count: 0,
            }
        }).unwrap();
        device.resume();
        device
    }

    pub fn queue(&mut self, left_sample: i16, right_sample: i16) {
        self.push(Audio::VOLUME_FACTOR * left_sample);
        self.push(Audio::VOLUME_FACTOR * right_sample);
    }

    fn push(&mut self, sample: i16) {
        self.buffer[self.write_i] = sample;
        self.write_i = (self.write_i + 1) % gba::AUDIO_BUFFER_LEN;
        self.count = std::cmp::min(self.count + 1, self.buffer.len());
    }

    fn pop(&mut self) -> i16 {
        let value = self.buffer[self.read_i];
        self.count -= 1;
        self.read_i = (self.read_i + 1) % gba::AUDIO_BUFFER_LEN;
        value
    }

    fn peek(&self, i: usize) -> i16 {
        self.buffer[(self.read_i + i) % gba::AUDIO_BUFFER_LEN]
    }
}

impl AudioCallback for Audio {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        if self.count < out.len() {
            for (i, x) in out.iter_mut().enumerate() {
                *x = self.peek(i % gba::AUDIO_BUFFER_LEN);
            }
        } else {
            for x in out.iter_mut() {
                *x = self.pop();
            }
        }
    }
}
