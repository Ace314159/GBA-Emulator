extern crate imgui;
extern crate imgui_memory_editor;

mod display;
mod debug;

use std::thread;

use core::flume;
use core::simplelog::*;
use core::gba::{GBA, VisibleMemoryRegion};
use display::Display;

use debug::TextureWindow;
use glfw::Key;
use imgui::*;
use imgui_memory_editor::MemoryEditor;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Error, Config::default(), TerminalMode::Mixed),
        TermLogger::new(LevelFilter::Off,
            ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .set_time_level(LevelFilter::Off)
            .set_max_level(LevelFilter::Off)
            .add_filter_allow_str("core::cpu")
            .build(),
            TerminalMode::Mixed),//std::fs::File::create("stdout.log").unwrap()),
    ]).unwrap();

    let (tx, rx) = flume::unbounded();
    let (mut gba, pixels_mutex) =
        GBA::new("Kirby - Nightmare in Dream Land (USA).gba".to_string(), tx);
    let gba_thread = thread::spawn(move || {
        loop { gba.emulate() }
    });
    let mut pixels_lock = None; 

    let mut imgui = Context::create();
    let mut display = Display::new(&mut imgui);
    let mut paused = false;

    /*let mut map_window = TextureWindow::new("BG Map");
    let mut tiles_window = TextureWindow::new("Tiles");
    let mut palettes_window = TextureWindow::new("Palettes");

    let map_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];
    let mut map_bg_i = 0;

    let tiles_block_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3"), im_str!("OBJ")];
    let mut tiles_block = 0;
    let mut tiles_bpp8 = false;
    let mut tiles_palette = 0;
    let mut mem_region = VisibleMemoryRegion::BIOS;
    let mut mem_editor = MemoryEditor::new(0)
        .read_fn(|addr| gba.peek_mem(mem_region, addr));*/

    while !display.should_close() {
        /*let (map_pixels, map_width, map_height) = gba.render_map(map_bg_i);
        let (tiles_pixels, tiles_width, tiles_height) =
            gba.render_tiles(tiles_palette as usize, tiles_block, tiles_bpp8);
        let (palettes_pixels, palettes_width, palettes_height) = gba.render_palettes();
        mem_editor = mem_editor
            .base_addr(mem_region.get_start_addr() as usize)
            .mem_size(mem_region.get_size());*/
        if !paused {
            rx.recv().unwrap();
            pixels_lock = Some(pixels_mutex.lock().unwrap());
        }
        
        let pixels = pixels_lock.take().unwrap();
        display.render(&pixels, &mut imgui, |ui, keys_pressed| {
            if keys_pressed.contains(&Key::P) { paused = !paused }
            if paused {
                Window::new(im_str!("Paused"))
                .no_decoration()
                .always_auto_resize(true)
                .build(ui, || {
                    ui.text("Paused");
                });
            }
            /*map_window.render(ui, &keys_pressed, map_pixels, map_width, map_height, || {
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
            let mut mem_region_i = mem_region as usize;
            Window::new(im_str!("Memory Viewer"))
            .build(ui, || {
                debug::control_combo_with_arrows(ui, &keys_pressed, &mut mem_region_i, 8);
                ComboBox::new(im_str!("Memory Region")).build_simple(ui, &mut mem_region_i,
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8],
                    &(|i| std::borrow::Cow::from(ImString::new(
                        VisibleMemoryRegion::from_index(*i).get_name()
                ))));
                mem_editor.build_without_window(&ui);
            });
            mem_region = VisibleMemoryRegion::from_index(mem_region_i);*/
        });

        if paused {
            pixels_lock = Some(pixels);
        }
    }
}
