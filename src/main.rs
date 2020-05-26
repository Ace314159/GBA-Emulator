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
    let mut gba = GBA::new("bin/bigmap.gba".to_string());

    let mut map_bg_i = 0;
    let mut map_scale = 1.0;
    let mut tiles_scale = 1.0;
    let scale_inc = 0.1;

    let mut tiles_bpp8 = false;

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.render_map(map_bg_i);
            let (tiles_pixels, tiles_width, tiles_height) = gba.render_tiles(tiles_bpp8);
            let map_texture = Texture::new(map_pixels, map_width, map_height);
            let tiles_texture = Texture::new(tiles_pixels, tiles_width, tiles_height);
            
            display.render(&mut gba, &mut imgui, |ui| {
                Window::new(im_str!("BG Map"))
                .always_auto_resize(true)
                .build(ui, || {
                    if ui.is_window_focused() {
                        if ui.io().keys_down[Key::Equal as usize] { map_scale += scale_inc }
                        if ui.io().keys_down[Key::Minus as usize] { map_scale -= scale_inc }
                    }
                    let labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];
                    ComboBox::new(im_str!("BG")).build_simple(ui, &mut map_bg_i,
                        &[0usize, 1, 2, 3,], &(|i| std::borrow::Cow::from(labels[*i])));

                    map_texture.render(map_scale).build(ui);
                });

                Window::new(im_str!("Tiles"))
                .always_auto_resize(true)
                .build(ui, || {
                    if ui.is_window_focused() {
                        if ui.io().keys_down[Key::Equal as usize] { tiles_scale += scale_inc }
                        if ui.io().keys_down[Key::Minus as usize] { tiles_scale -= scale_inc }
                    }
                    ui.checkbox(im_str!("256 colors"), &mut tiles_bpp8);

                    tiles_texture.render(tiles_scale).build(ui);
                });
            });
            
        }
    }
}
