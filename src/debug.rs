use imgui::*;

pub struct Texture {
    tex: u32,
    width: f32,
    height: f32,
}

impl Texture {
    pub fn new() -> Texture {
        Texture {
            tex: 0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn update_pixels(&mut self, pixels: Vec<u16>, width: usize, height: usize) {
        let width = width as f32;
        let height = height as f32;
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            unsafe {
                gl::DeleteTextures(1, &mut self.tex as *mut u32);
                gl::GenTextures(1, &mut self.tex as *mut u32);
                gl::BindTexture(gl::TEXTURE_2D, self.tex);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA8, self.width as i32, self.height as i32);
            }
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.tex);
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, self.width as i32, self.height as i32,
                gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV, pixels.as_ptr() as *const std::ffi::c_void);
        }
    }

    pub fn render(&self, scale: f32) -> Image {
        Image::new(TextureId::from(self.tex as usize), [self.width * scale, self.height * scale])
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &mut self.tex as *mut u32) }
    }
}
