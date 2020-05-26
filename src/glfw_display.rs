extern crate imgui_glfw_rs;

use imgui_glfw_rs::{
    imgui::{Context as ImContext, Ui},
    ImguiGLFW,
    glfw::{self, Action, Context, Glfw, Key, Window}
};
use std::time::SystemTime;

use core::gba::{self, GBA, KEYINPUT};

pub struct GLFWDisplay {
    imgui_glfw: ImguiGLFW,
    imgui: ImContext,
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

        let width = (gba::WIDTH * gba::SCALE) as u32;
        let height = (gba::HEIGHT * gba::SCALE) as u32;
        let (mut window, events) = glfw.create_window(width, height, "GBA Emulator", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_current();
        window.set_all_polling(true);

        let mut imgui = ImContext::create();

        let imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

        let mut screen_tex = 0u32;
        let mut fbo = 0u32;
        let color_black = [1f32, 0f32, 0f32];
        
        gl::load_with(|name| window.get_proc_address(name));
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null_mut());
            
            gl::GenTextures(1, &mut screen_tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, screen_tex);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, &color_black as *const f32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA8, gba::WIDTH as i32, gba::HEIGHT as i32);
            
            gl::GenFramebuffers(1, &mut fbo as *mut u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_tex, 0);
        }

         GLFWDisplay {
            imgui_glfw,
            imgui,
            glfw,
            window,
            events,

            prev_frame_time: SystemTime::now(),
            prev_fps_update_time: SystemTime::now(),
            frames_passed: 0,
        }
    }

    pub fn should_close(&self) -> bool { self.window.should_close() }

    pub fn render<F>(&mut self, gba: &mut GBA, imgui_draw: F) where F: FnOnce(&mut Ui) {
        let pixels = gba.get_pixels();
        let (width, height) = self.window.get_size();

        let (tex_x, tex_y) = if width * gba::HEIGHT as i32 > height * gba::WIDTH as i32 {
            let scaled_width = (gba::WIDTH as f32 / gba::HEIGHT as f32 * height as f32) as i32;
            ((width - scaled_width) / 2, 0)
        } else if width * (gba::HEIGHT as i32) < height * gba::WIDTH as i32 {
            let scaled_height = (gba::HEIGHT as f32 / gba::WIDTH as f32 * width as f32) as i32;
            (0, (height - scaled_height) / 2)
        } else { (0, 0) };

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, gba::WIDTH as i32, gba::HEIGHT as i32,
                gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, pixels.as_ptr() as *const std::ffi::c_void);
            gl::BlitFramebuffer(0, 0, gba::WIDTH as i32, gba::HEIGHT as i32,
                tex_x, height - tex_y, width - tex_x, tex_y, gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        let mut ui = self.imgui_glfw.frame(&mut self.window, &mut self.imgui);
        imgui_draw(&mut ui);
        self.imgui_glfw.draw(ui, &mut self.window);
        self.window.swap_buffers();
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            self.imgui_glfw.handle_event(&mut self.imgui, &event);
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
