extern crate sdl2;

mod sdl_screen;

use sdl2::event::{Event, WindowEvent};

use core::gba::{GBA, Screen};
use sdl_screen::SDLScreen;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let sdl_ctx = sdl2::init().unwrap();
    let mut screen = SDLScreen::new(&sdl_ctx);
    
    let mut gba = GBA::new(&"armwrestler.gba".to_string());

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    'running: loop {
        gba.emulate();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                    screen.set_size(width, height)}
                _ => {},
            }
        }
    }
}
