extern crate imgui;

mod display;
mod debug;

use core::gba::GBA;
use display::Display;

use debug::TextureWindow;
use imgui::*;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let mut imgui = Context::create();
    let mut display = Display::new(&mut imgui);
    let mut gba = GBA::new("Pokemon Pinball - Ruby & Sapphire (USA).gba".to_string());

    let mut map_window = TextureWindow::new("BG Map");
    let mut tiles_window = TextureWindow::new("BG Tiles");
    let mut palettes_window = TextureWindow::new("Palettes");

    let mut map_bg_i = 0;
    let mut tiles_bpp8 = false;

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.render_map(map_bg_i);
            let (tiles_pixels, tiles_width, tiles_height) = gba.render_tiles(tiles_bpp8);
            let (palettes_pixels, palettes_width, palettes_height) = gba.render_palettes();

            display.render(&mut gba, &mut imgui, |ui| {
                map_window.render(ui, map_pixels, map_width, map_height, || {
                    let labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];
                    ComboBox::new(im_str!("BG")).build_simple(ui, &mut map_bg_i,
                        &[0usize, 1, 2, 3,], &(|i| std::borrow::Cow::from(labels[*i])));
                });
                tiles_window.render(ui, tiles_pixels, tiles_width, tiles_height, || {
                    ui.checkbox(im_str!("256 colors"), &mut tiles_bpp8);
                });
                palettes_window.render(ui, palettes_pixels, palettes_width, palettes_height, || {});
            });
            
        }
    }
}
