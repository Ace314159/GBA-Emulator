extern crate glfw;

mod glfw_display;

use core::gba::{GBA, Display};
use glfw_display::GLFWDisplay;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let mut display = GLFWDisplay::new();
    
    let mut gba = GBA::new(&"armwrestler.gba".to_string());

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            display.render(&mut gba);
        }
    }
}
