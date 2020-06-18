mod components;
pub use components::Timer;

use super::IORegister;

mod tone;
mod wave;
mod noise;
mod dma_sound;

pub use tone::Tone;
pub use wave::Wave;
pub use noise::Noise;
pub use dma_sound::DMASound;

pub trait Channel {
    fn generate_sample(&self) -> i16;
    fn is_on(&self) -> bool;
}
