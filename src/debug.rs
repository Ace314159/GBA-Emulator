use imgui_glfw_rs::imgui::*;

pub struct Texture {
    tex: u32,
    width: f32,
    height: f32,
}

impl Texture {
    pub fn new(pixels: Vec<u16>, width: usize, height: usize) -> Texture {
        let mut tex = 0u32;
        unsafe {
            gl::GenTextures(1, &mut tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0,
                gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, pixels.as_ptr() as *const std::ffi::c_void);
        }

        Texture {
            tex,
            width: width as f32,
            height: height as f32,
        }
    }

    pub fn render(&self, ui: &Ui, scale: f32) {
        Image::new(ui, TextureId::from(self.tex as usize), [self.width * scale, self.height * scale]).build();
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &mut self.tex as *mut u32) }
    }
}
