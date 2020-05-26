extern crate imgui;

mod display;
mod debug;

use core::gba::GBA;
use display::Display;

use debug::Texture;
use imgui::*;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let mut imgui = Context::create();
    let mut display = Display::new(&mut imgui);
    let mut gba = GBA::new("bin/bigmap.gba".to_string());

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.get_map(0);
            let map_scale = 3.0;
            let map_texture = Texture::new(map_pixels, map_width, map_height);
            
            display.render(&mut gba, &mut imgui, |ui| {
                Window::new(im_str!("BG Map"))
                .size([map_width as f32 * map_scale, map_height as f32 * map_scale], Condition::FirstUseEver)
                .build(ui, || {
                    map_texture.render(map_scale).build(ui);
                });
            });
            
        }
    }
}
