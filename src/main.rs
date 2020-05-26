extern crate imgui_glfw_rs;

mod glfw_display;
mod debug;

use core::gba::GBA;
use glfw_display::GLFWDisplay;

use debug::Texture;
use imgui_glfw_rs::imgui::*;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let mut display = GLFWDisplay::new();
    
    let mut gba = GBA::new("bin/bigmap.gba".to_string());

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.get_map(0);
            let map_scale = 3.0;
            let map_texture = Texture::new(map_pixels, map_width, map_height);
            
            display.render(&mut gba, |ui| {
                Window::new(ui, im_str!("BG Map"))
                .size([map_width as f32 * map_scale, map_height as f32 * map_scale], Condition::FirstUseEver)
                .build(|| {
                    map_texture.render(ui, map_scale);
                });
            });
            
        }
    }
}
