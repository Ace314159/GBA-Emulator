extern crate sdl2;

mod sdl_screen;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;

use core::gba::{GBA, Screen, KEYINPUT};
use sdl_screen::SDLScreen;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();

    let sdl_ctx = sdl2::init().unwrap();
    let mut screen = SDLScreen::new(&sdl_ctx);
    
    let mut gba = GBA::new(&"armwrestler.gba".to_string());

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    'running: loop {
        gba.emulate();
        if gba.needs_to_render() {
            screen.render(gba.get_pixels());
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                        screen.set_size(width, height)}
                    Event::KeyDown { keycode: Some(Keycode::A), ..} => gba.press_key(KEYINPUT::A),
                    Event::KeyDown { keycode: Some(Keycode::B), ..} => gba.press_key(KEYINPUT::B),
                    Event::KeyDown { keycode: Some(Keycode::E), ..} => gba.press_key(KEYINPUT::SELECT),
                    Event::KeyDown { keycode: Some(Keycode::T), ..} => gba.press_key(KEYINPUT::START),
                    Event::KeyDown { keycode: Some(Keycode::Right), ..} => gba.press_key(KEYINPUT::RIGHT),
                    Event::KeyDown { keycode: Some(Keycode::Left), ..} => gba.press_key(KEYINPUT::LEFT),
                    Event::KeyDown { keycode: Some(Keycode::Up), ..} => gba.press_key(KEYINPUT::UP),
                    Event::KeyDown { keycode: Some(Keycode::Down), ..} => gba.press_key(KEYINPUT::DOWN),
                    Event::KeyDown { keycode: Some(Keycode::R), ..} => gba.press_key(KEYINPUT::RIGHT),
                    Event::KeyDown { keycode: Some(Keycode::L), ..} => gba.press_key(KEYINPUT::LEFT),
                    Event::KeyUp { keycode: Some(Keycode::A), ..} => gba.release_key(KEYINPUT::A),
                    Event::KeyUp { keycode: Some(Keycode::B), ..} => gba.release_key(KEYINPUT::B),
                    Event::KeyUp { keycode: Some(Keycode::E), ..} => gba.release_key(KEYINPUT::SELECT),
                    Event::KeyUp { keycode: Some(Keycode::T), ..} => gba.release_key(KEYINPUT::START),
                    Event::KeyUp { keycode: Some(Keycode::Right), ..} => gba.release_key(KEYINPUT::RIGHT),
                    Event::KeyUp { keycode: Some(Keycode::Left), ..} => gba.release_key(KEYINPUT::LEFT),
                    Event::KeyUp { keycode: Some(Keycode::Up), ..} => gba.release_key(KEYINPUT::UP),
                    Event::KeyUp { keycode: Some(Keycode::Down), ..} => gba.release_key(KEYINPUT::DOWN),
                    Event::KeyUp { keycode: Some(Keycode::R), ..} => gba.release_key(KEYINPUT::RIGHT),
                    Event::KeyUp { keycode: Some(Keycode::L), ..} => gba.release_key(KEYINPUT::LEFT),
                    _ => {},
                }
            }
        }

    }
}
