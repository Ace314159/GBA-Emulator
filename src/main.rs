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
    let mut tiles_window = TextureWindow::new("Tiles");
    let mut palettes_window = TextureWindow::new("Palettes");

    let map_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];
    let mut map_bg_i = 0;

    let tiles_block_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3"), im_str!("OBJ")];
    let mut tiles_block = 0;
    let mut tiles_bpp8 = false;
    let mut tiles_palette = 0;

    while !display.should_close() {
        gba.emulate();
        if gba.needs_to_render() {
            let (map_pixels, map_width, map_height) = gba.render_map(map_bg_i);
            let (tiles_pixels, tiles_width, tiles_height) =
                gba.render_tiles(tiles_palette as usize, tiles_block, tiles_bpp8);
            let (palettes_pixels, palettes_width, palettes_height) = gba.render_palettes();

            display.render(&mut gba, &mut imgui, |ui, keys_pressed| {
                map_window.render(ui, map_pixels, map_width, map_height, || {
                map_window.render(ui, &keys_pressed, map_pixels, map_width, map_height, || {
                    debug::control_combo_with_arrows(ui, &keys_pressed,  &mut map_bg_i, map_labels.len() - 1);
                    ComboBox::new(im_str!("BG")).build_simple(ui, &mut map_bg_i,
                        &[0usize, 1, 2, 3], &(|i| std::borrow::Cow::from(map_labels[*i])));
                });
                tiles_window.render(ui, &keys_pressed, tiles_pixels, tiles_width, tiles_height, || {
                    debug::control_combo_with_arrows(ui, &keys_pressed, &mut tiles_block, tiles_block_labels.len() - 1);
                    ComboBox::new(im_str!("Block")).build_simple(ui, &mut tiles_block,
                        &[0, 1, 2, 3, 4], &(|i| std::borrow::Cow::from(tiles_block_labels[*i])));
                    ui.checkbox(im_str!("256 colors"), &mut tiles_bpp8);
                    if !tiles_bpp8 {
                        ui.input_int(im_str!("Palette"), &mut tiles_palette)
                        .step(1)
                        .build();
                        tiles_palette = if tiles_palette > 15 { 15 } else if tiles_palette < 0 { 0 } else { tiles_palette };
                    }
                });
                palettes_window.render(ui, &keys_pressed, palettes_pixels, palettes_width, palettes_height, || {});
            });
            
        }
    }
}
