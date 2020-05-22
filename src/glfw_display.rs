extern crate glfw;

use glfw::{Action, Context, Glfw, Key, Window};
use std::time::SystemTime;

use core::gba::{GBA, Display, KEYINPUT};

pub struct GLFWDisplay {
    glfw: Glfw,
    window: Window,
    events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,

    prev_frame_time: SystemTime,
    prev_fps_update_time: SystemTime,
    frames_passed: u32,
}

impl GLFWDisplay {
    pub fn new() -> GLFWDisplay {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

        let width = (Display::WIDTH * Display::SCALE) as u32;
        let height = (Display::HEIGHT * Display::SCALE) as u32;
        let (mut window, events) = glfw.create_window(width, height, "GBA Emulator", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        gl::load_with(|name| window.get_proc_address(name));

        window.make_current();
        window.set_key_polling(true);

        let mut screen_tex = 0u32;
        let mut fbo = 0u32;
        let color_black = [1f32, 0f32, 0f32];

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null_mut());
            
            gl::GenTextures(1, &mut screen_tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, screen_tex);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, &color_black as *const f32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA8, Display::WIDTH as i32, Display::HEIGHT as i32);
            
            gl::GenFramebuffers(1, &mut fbo as *mut u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_tex, 0);
        }

        GLFWDisplay {
            glfw,
            window,
            events,

            prev_frame_time: SystemTime::now(),
            prev_fps_update_time: SystemTime::now(),
            frames_passed: 0,
        }
    }
}

impl Display for GLFWDisplay {
    fn should_close(&self) -> bool { self.window.should_close() }

    fn render(&mut self, gba: &mut GBA) {
        let pixels = gba.get_pixels();
        let (width, height) = self.window.get_size();

        let (tex_x, tex_y) = if width * Display::HEIGHT as i32 > height * Display::WIDTH as i32 {
            let scaled_width = (Display::WIDTH as f32 / Display::HEIGHT as f32 * height as f32) as i32;
            ((width - scaled_width) / 2, 0)
        } else if width * (Display::HEIGHT as i32) < height * Display::WIDTH as i32 {
            let scaled_height = (Display::HEIGHT as f32 / Display::WIDTH as f32 * width as f32) as i32;
            (0, (height - scaled_height) / 2)
        } else { (0, 0) };

        unsafe {
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, Display::WIDTH as i32, Display::HEIGHT as i32,
                gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, pixels.as_ptr() as *const std::ffi::c_void);
            gl::BlitFramebuffer(0, 0, Display::WIDTH as i32, Display::HEIGHT as i32,
                tex_x, height - tex_y, width - tex_x, tex_y, gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        self.window.swap_buffers();
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(key, _, action, _) => {
                    let keypad_key = match key {
                        Key::A => KEYINPUT::A,
                        Key::B => KEYINPUT::B,
                        Key::E => KEYINPUT::SELECT,
                        Key::T => KEYINPUT::START,
                        Key::Right => KEYINPUT::RIGHT,
                        Key::Left => KEYINPUT::LEFT,
                        Key::Up => KEYINPUT::UP,
                        Key::Down => KEYINPUT::DOWN,
                        Key::R => KEYINPUT::R,
                        Key::L => KEYINPUT::L,
                        _ => continue,
                    };
                    match action {
                        Action::Press => gba.press_key(keypad_key),
                        Action::Release => gba.release_key(keypad_key),
                        _ => continue,
                    };
                },
                _ => {},
            }
        }
        self.prev_frame_time = SystemTime::now();

        self.frames_passed += 1;
        let cur_time = SystemTime::now();
        let time_passed = cur_time.duration_since(self.prev_fps_update_time).unwrap().as_secs_f64();
        if time_passed >= 1.0 {
            let fps = self.frames_passed as f64 / time_passed;
            self.window.set_title(&format!("GBA Emulator - {:.2} FPS", fps));
            self.frames_passed = 0;
            self.prev_fps_update_time = cur_time;
        }
    }
}

extern "system"
fn gl_debug_callback(_source: u32, _type: u32, _id: u32, sev: u32, _len: i32,
    message: *const i8, _param: *mut std::ffi::c_void) {
    if sev == gl::DEBUG_SEVERITY_NOTIFICATION { return }

    unsafe {
        let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
        panic!("OpenGL Debug message: {}", message);
    }
}
