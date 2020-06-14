extern crate sdl2;

use sdl2::audio::{AudioSpecDesired, AudioQueue};

use crate::gba;

pub struct Audio {
    queue: AudioQueue<f32>,
}

impl Audio {
    const DESIRED_SPEC: AudioSpecDesired = AudioSpecDesired {
        freq: Some(gba::AUDIO_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: None,
    };

    pub fn new() -> Audio {
        let sdl_ctx = sdl2::init().unwrap();
        let audio_subsystem = sdl_ctx.audio().unwrap();

        let queue = audio_subsystem.open_queue(None, &Audio::DESIRED_SPEC).unwrap();
        queue.resume();
        Audio {
            queue,
        }
    }

    pub fn queue(&self, left_sample: f32, right_sample: f32) {
        self.queue.queue(&[left_sample, right_sample]);
    }
}
