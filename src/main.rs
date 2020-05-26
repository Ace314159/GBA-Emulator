extern crate imgui;

mod display;
mod debug;

use core::gba::GBA;
use display::Display;

use debug::Texture;
use glfw::Key;
use imgui::*;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let mut imgui = Context::create();
    let mut display = Display::new(&mut imgui);
    let mut gba = GBA::new("bin/sbb_aff.gba".to_string());
    let mut map_bg_i = 0;
    let mut map_scale = 1.0;
    let scale_inc = 0.1;
    

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.render_map(map_bg_i);
            let map_texture = Texture::new(map_pixels, map_width, map_height);
            
            display.render(&mut gba, &mut imgui, |ui| {
                if ui.io().keys_down[Key::Equal as usize] { map_scale += scale_inc }
                if ui.io().keys_down[Key::Minus as usize] { map_scale -= scale_inc }
                Window::new(im_str!("BG Map"))
                .always_auto_resize(true)
                .build(ui, || {
                    let labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];

                    ComboBox::new(im_str!("BG")).build_simple(ui, &mut map_bg_i,
                        &[0usize, 1, 2, 3,], &(|i| std::borrow::Cow::from(labels[*i])));
                    map_texture.render(map_scale).build(ui);
                });
            });
            
        }
    }
}
