use nalgebra as na;
use failure;
use gl;
use gl::types::*;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Render target creation failed. The framebuffer was not complete.")]
    TheFramebufferWasNotComplete,
}

pub struct DefaultRenderTarget {
    size: na::Vector2<i32>,
    gl: gl::Gl,
}

impl DefaultRenderTarget {
    pub fn new(gl: &gl::Gl, size: na::Vector2<i32>) -> DefaultRenderTarget {
        DefaultRenderTarget {
            size,
            gl: gl.clone(),
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

pub struct FramebufferTarget {
    size: na::Vector2<i32>,
    fb: GLuint,
    tex: GLuint,
    gl: gl::Gl,
}

impl FramebufferTarget {
    pub fn new(gl: &gl::Gl, size: na::Vector2<i32>) -> Result<FramebufferTarget, Error> {
        let mut tex = 0;
        let mut fb = 0;

        unsafe {
            gl.GenFramebuffers(1, &mut fb);
            gl.BindFramebuffer(gl::FRAMEBUFFER, fb);

            gl.GenTextures(1, &mut tex);
            gl.BindTexture(gl::TEXTURE_2D, tex);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

            gl.FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, tex, 0);

            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, size.x, size.y, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null());

            let complete = gl.CheckFramebufferStatus(gl::FRAMEBUFFER);

            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl.BindTexture(gl::TEXTURE_2D, 0);

            if complete != gl::FRAMEBUFFER_COMPLETE {
                return Err(Error::TheFramebufferWasNotComplete);
            }
        }

        Ok(FramebufferTarget {
            size,
            fb,
            tex,
            gl: gl.clone(),
        })
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, self.fb);
        }
    }
}

impl Drop for FramebufferTarget
{
    fn drop(&mut self){
        unsafe{
            self.gl.DeleteFramebuffers(1, &self.fb);
            self.gl.DeleteTextures(1, &self.tex);
        }
    }
}