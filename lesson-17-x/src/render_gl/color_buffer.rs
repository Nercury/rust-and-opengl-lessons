use nalgebra as na;
use gl;

pub struct ColorBuffer;

impl ColorBuffer {
    pub fn new() -> ColorBuffer {
        ColorBuffer
    }

    pub fn set_clear_color(&self, gl: &gl::Gl, color: na::Vector3<f32>) {
        unsafe {
            gl.ClearColor(color.x, color.y, color.z, 1.0);
        }
    }

    pub fn set_default_blend_func(&self, gl: &gl::Gl) {
        unsafe {
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn enable_blend(&self, gl: &gl::Gl) {
        unsafe {
            gl.Enable(gl::BLEND);
        }
    }

    pub fn disable_blend(&self, gl: &gl::Gl) {
        unsafe {
            gl.Disable(gl::BLEND);
        }
    }
}