extern crate imgui;
// extern crate imgui_memory_editor;

mod display;
mod debug;

use std::thread;
use std::collections::VecDeque;

use core::flume;
use core::simplelog::*;
//use core::gba::{GBA, VisibleMemoryRegion};
use core::gba::GBA;
use display::Display;

use debug::TextureWindow;
use glfw::Key;
use imgui::*;
//use imgui_memory_editor::MemoryEditor;

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

    let (render_tx, render_rx) = flume::unbounded();
    let (keypad_tx, keypad_rx) = flume::unbounded();
    let (mutexes_tx, mutexes_rx) = flume::unbounded();
    let _gba_thread = thread::spawn(move || {
        let (mut gba, pixels_mutex, debug_windows_spec_mutex) =
        GBA::new("Pokemon - Emerald Version (USA, Europe).gba".to_string(), render_tx, keypad_rx);
        mutexes_tx.send((pixels_mutex, debug_windows_spec_mutex)).unwrap();
        loop { gba.emulate() }
    });
    let (pixels_mutex, debug_windows_spec_mutex) = mutexes_rx.recv().unwrap();
    let mut pixels_lock = None; 

    let mut imgui = Context::create();
    let mut display = Display::new(&mut imgui);
    let mut paused = false;

    let mut map_window = TextureWindow::new("BG Map");
    let mut tiles_window = TextureWindow::new("Tiles");
    let mut palettes_window = TextureWindow::new("Palettes");

    let map_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3")];
    let tiles_block_labels = [im_str!("0"), im_str!("1"), im_str!("2"), im_str!("3"), im_str!("OBJ")];

    let mut debug_windows = VecDeque::new();
    /*let mut mem_region = VisibleMemoryRegion::BIOS;
    let mut mem_editor = MemoryEditor::new(0)
        .read_fn(|addr| gba.peek_mem(mem_region, addr));*/

    while !display.should_close() {
        /*mem_editor = mem_editor
            .base_addr(mem_region.get_start_addr() as usize)
            .mem_size(mem_region.get_size());*/
        if !paused {
            debug_windows = render_rx.recv().unwrap();
            pixels_lock = Some(pixels_mutex.lock().unwrap());
        }
        
        let pixels = pixels_lock.take().unwrap();
        let mut debug_windows_spec = debug_windows_spec_mutex.lock().unwrap();
        let mut debug_windows_copy = debug_windows.clone();
        display.render(&pixels, &keypad_tx, &mut imgui,
            |ui, keys_pressed, modifers| {
            if paused {
                Window::new(im_str!("Paused"))
                .no_decoration()
                .always_auto_resize(true)
                .build(ui, || {
                    ui.text("Paused");
                });
            }
            if debug_windows_spec.map_enable {
                let (pixels, width, height) = debug_windows_copy.pop_front().unwrap();
                let bg_i = &mut debug_windows_spec.map_spec.bg_i;
                map_window.render(ui, &keys_pressed, pixels, width, height, || {
                    debug::control_combo_with_arrows(ui, &keys_pressed, bg_i, map_labels.len() - 1);
                    ComboBox::new(im_str!("BG")).build_simple(ui, bg_i,
                        &[0usize, 1, 2, 3], &(|i| std::borrow::Cow::from(map_labels[*i])));
                });
            }
            if debug_windows_spec.tiles_enable {
                let (pixels, width, height) = debug_windows_copy.pop_front().unwrap();
                let spec = &mut debug_windows_spec.tiles_spec;
                let (palette, block, bpp8) = (&mut spec.palette, &mut spec.block, &mut spec.bpp8);
                tiles_window.render(ui, &keys_pressed, pixels, width, height, || {
                    debug::control_combo_with_arrows(ui, &keys_pressed, block, tiles_block_labels.len() - 1);
                    ComboBox::new(im_str!("Block")).build_simple(ui, block,
                        &[0, 1, 2, 3, 4], &(|i| std::borrow::Cow::from(tiles_block_labels[*i])));
                    ui.checkbox(im_str!("256 colors"), bpp8);
                    if !*bpp8 {
                        ui.input_int(im_str!("Palette"), palette)
                        .step(1)
                        .build();
                        *palette = if *palette > 15 { 15 } else if *palette < 0 { 0 } else { *palette };
                    }
                });
            }
            if debug_windows_spec.palettes_enable {
                let (pixels, width, height) = debug_windows_copy.pop_front().unwrap();
                palettes_window.render(ui, &keys_pressed, pixels, width, height, || {});
            }
            /*let mut mem_region_i = mem_region as usize;
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

            if modifers.contains(&glfw::Modifiers::Control) {
                if paused { return }
                if keys_pressed.contains(&Key::M) { debug_windows_spec.map_enable = !debug_windows_spec.map_enable }
                if keys_pressed.contains(&Key::T) { debug_windows_spec.tiles_enable = !debug_windows_spec.tiles_enable }
                if keys_pressed.contains(&Key::P) { debug_windows_spec.palettes_enable = !debug_windows_spec.palettes_enable }
            } else if keys_pressed.contains(&Key::P) { paused = !paused }
        });
        drop(debug_windows_spec);

        if paused {
            pixels_lock = Some(pixels);
        }
    }
}
