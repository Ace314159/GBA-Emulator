extern crate sdl2;

use sdl2::video::{GLContext, Window, GLProfile, SwapInterval};
use std::time::SystemTime;

use core::gba::Screen;

pub struct SDLScreen {
    _gl_ctx: GLContext,
    window: Window,

    prev_frame_time: SystemTime,
    prev_fps_update_time: SystemTime,
    frames_passed: u32,

    width: i32,
    height: i32,

    _screen_tex: u32,
    fbo: u32,
}

impl SDLScreen {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> SDLScreen {
        let video_subsystem = sdl_ctx.video().unwrap();
        
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let width = (Screen::WIDTH * Screen::SCALE) as u32;
        let height = (Screen::HEIGHT * Screen::SCALE) as u32;
        let window = video_subsystem.window("GBA Emulator", width, height).resizable().opengl().build().unwrap();

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        video_subsystem.gl_set_swap_interval(SwapInterval::Immediate).unwrap();

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let mut screen_tex = 0u32;
        let mut fbo = 0u32; 
        let color_black = [0f32, 0f32, 0f32];

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null_mut());

            gl::GenTextures(1, &mut screen_tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, screen_tex);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, &color_black as *const f32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, Screen::WIDTH as i32,
                Screen::HEIGHT as i32, 0, gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, std::ptr::null_mut());

            gl::GenFramebuffers(1, &mut fbo as *mut u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_tex, 0);
        }

        SDLScreen {
            _gl_ctx: gl_ctx,
            window,

            prev_frame_time: SystemTime::now(),
            prev_fps_update_time: SystemTime::now(),
            frames_passed: 0,

            width: width as i32,
            height: height as i32,

            _screen_tex: screen_tex,
            fbo,
        }
    }
}

impl Screen for SDLScreen {
    fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn render(&mut self, pixels: &[u16; Screen::WIDTH * Screen::HEIGHT]) {
        let (tex_x, tex_y) = if self.width * Screen::HEIGHT as i32 > self.height * Screen::WIDTH as i32 {
            let scaled_width = (Screen::WIDTH as f32 / Screen::HEIGHT as f32 * self.height as f32) as i32;
            ((self.width - scaled_width) / 2, 0)
        } else if self.width * (Screen::HEIGHT as i32) < self.height * Screen::WIDTH as i32 {
            let scaled_height = (Screen::HEIGHT as f32 / Screen::WIDTH as f32 * self.width as f32) as i32;
            (0, (self.height - scaled_height) / 2)
        } else { (0, 0) };

        unsafe {
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, Screen::WIDTH as i32, Screen::HEIGHT as i32,
                gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, pixels.as_ptr() as *const std::ffi::c_void);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::BlitFramebuffer(0, 0, Screen::WIDTH as i32, Screen::HEIGHT as i32,
                tex_x, self.height - tex_y, self.width - tex_x, tex_y, gl::COLOR_BUFFER_BIT, gl::NEAREST);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }

        self.window.gl_swap_window();
        self.prev_frame_time = SystemTime::now();

        self.frames_passed += 1;
        let cur_time = SystemTime::now();
        let time_passed = cur_time.duration_since(self.prev_fps_update_time).unwrap().as_secs_f64();
        if time_passed >= 1.0 {
            let fps = self.frames_passed as f64 / time_passed;
            self.window.set_title(&format!("GBA Emulator - {:.2} FPS", fps)).unwrap();
            self.frames_passed = 0;
            self.prev_fps_update_time = cur_time;
        }
    }
}

extern "system"
fn gl_debug_callback(_source: u32, _type: u32, _id: u32, _sev: u32, _len: i32,
    message: *const i8, _param: *mut std::ffi::c_void) {
    
    unsafe {
        let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
        panic!("OpenGL Debug message: {}", message);
    }
}
