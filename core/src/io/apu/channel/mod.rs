mod components;

use super::IORegister;

mod tone;
pub use tone::Tone;

pub trait Channel {
    fn generate_sample(&self) -> f32;
}
