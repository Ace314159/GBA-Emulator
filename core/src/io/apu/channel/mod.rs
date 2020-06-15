mod components;
pub use components::Timer;

use super::IORegister;

mod tone;
mod wave;

pub use tone::Tone;
pub use wave::Wave;

pub trait Channel {
    fn generate_sample(&self) -> i16;
    fn is_on(&self) -> bool;
}
