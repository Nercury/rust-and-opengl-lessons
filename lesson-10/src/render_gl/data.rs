use gl;

#[allow(non_camel_case_types)]
#[repr(C, packed)]
pub struct f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
}

impl f32_f32_f32 {
    pub fn new(d0: f32, d1: f32, d2: f32) -> f32_f32_f32 {
        f32_f32_f32 {
            d0, d1, d2
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f32, f32, f32)> for f32_f32_f32 {
    fn from(other: (f32, f32, f32)) -> Self {
        f32_f32_f32::new(other.0, other.1, other.2)
    }
}