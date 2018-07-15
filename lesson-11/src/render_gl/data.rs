#![allow(non_camel_case_types)]

use gl;

#[repr(C, packed)]
pub struct i8_ {
    pub d0: i8,
}

impl i8_ {
    pub fn new(d0: i8) -> i8_ {
        i8_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<i8> for i8_ {
    fn from(other: i8) -> Self {
        i8_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8 {
    pub d0: i8,
    pub d1: i8,
}

impl i8_i8 {
    pub fn new(d0: i8, d1: i8) -> i8_i8 {
        i8_i8 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8)> for i8_i8 {
    fn from(other: (i8, i8)) -> Self {
        i8_i8::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8_i8 {
    pub d0: i8,
    pub d1: i8,
    pub d2: i8,
}

impl i8_i8_i8 {
    pub fn new(d0: i8, d1: i8, d2: i8) -> i8_i8_i8 {
        i8_i8_i8 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8, i8)> for i8_i8_i8 {
    fn from(other: (i8, i8, i8)) -> Self {
        i8_i8_i8::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8_i8_i8 {
    pub d0: i8,
    pub d1: i8,
    pub d2: i8,
    pub d3: i8,
}

impl i8_i8_i8_i8 {
    pub fn new(d0: i8, d1: i8, d2: i8, d3: i8) -> i8_i8_i8_i8 {
       i8_i8_i8_i8 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8, i8, i8)> for i8_i8_i8_i8 {
    fn from(other: (i8, i8, i8, i8)) -> Self {
        i8_i8_i8_i8::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_float {
    pub d0: i8,
}

impl i8_float {
    pub fn new(d0: i8) -> i8_float {
        i8_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<i8> for i8_float {
    fn from(other: i8) -> Self {
        i8_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8_float {
    pub d0: i8,
    pub d1: i8,
}

impl i8_i8_float {
    pub fn new(d0: i8, d1: i8) -> i8_i8_float {
        i8_i8_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8)> for i8_i8_float {
    fn from(other: (i8, i8)) -> Self {
        i8_i8_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8_i8_float {
    pub d0: i8,
    pub d1: i8,
    pub d2: i8,
}

impl i8_i8_i8_float {
    pub fn new(d0: i8, d1: i8, d2: i8) -> i8_i8_i8_float {
        i8_i8_i8_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8, i8)> for i8_i8_i8_float {
    fn from(other: (i8, i8, i8)) -> Self {
        i8_i8_i8_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i8_i8_i8_i8_float {
    pub d0: i8,
    pub d1: i8,
    pub d2: i8,
    pub d3: i8,
}

impl i8_i8_i8_i8_float {
    pub fn new(d0: i8, d1: i8, d2: i8, d3: i8) -> i8_i8_i8_i8_float {
        i8_i8_i8_i8_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i8, i8, i8, i8)> for i8_i8_i8_i8_float {
    fn from(other: (i8, i8, i8, i8)) -> Self {
        i8_i8_i8_i8_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_ {
    pub d0: i16,
}

impl i16_ {
    pub fn new(d0: i16) -> i16_ {
        i16_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<i16> for i16_ {
    fn from(other: i16) -> Self {
        i16_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16 {
    pub d0: i16,
    pub d1: i16,
}

impl i16_i16 {
    pub fn new(d0: i16, d1: i16) -> i16_i16 {
        i16_i16 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16)> for i16_i16 {
    fn from(other: (i16, i16)) -> Self {
        i16_i16::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16_i16 {
    pub d0: i16,
    pub d1: i16,
    pub d2: i16,
}

impl i16_i16_i16 {
    pub fn new(d0: i16, d1: i16, d2: i16) -> i16_i16_i16 {
        i16_i16_i16 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16, i16)> for i16_i16_i16 {
    fn from(other: (i16, i16, i16)) -> Self {
        i16_i16_i16::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16_i16_i16 {
    pub d0: i16,
    pub d1: i16,
    pub d2: i16,
    pub d3: i16,
}

impl i16_i16_i16_i16 {
    pub fn new(d0: i16, d1: i16, d2: i16, d3: i16) -> i16_i16_i16_i16 {
        i16_i16_i16_i16 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16, i16, i16)> for i16_i16_i16_i16 {
    fn from(other: (i16, i16, i16, i16)) -> Self {
        i16_i16_i16_i16::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_float {
    pub d0: i16,
}

impl i16_float {
    pub fn new(d0: i16) -> i16_float {
        i16_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<i16> for i16_float {
    fn from(other: i16) -> Self {
        i16_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16_float {
    pub d0: i16,
    pub d1: i16,
}

impl i16_i16_float {
    pub fn new(d0: i16, d1: i16) -> i16_i16_float {
        i16_i16_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16)> for i16_i16_float {
    fn from(other: (i16, i16)) -> Self {
        i16_i16_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16_i16_float {
    pub d0: i16,
    pub d1: i16,
    pub d2: i16,
}

impl i16_i16_i16_float {
    pub fn new(d0: i16, d1: i16, d2: i16) -> i16_i16_i16_float {
        i16_i16_i16_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16, i16)> for i16_i16_i16_float {
    fn from(other: (i16, i16, i16)) -> Self {
        i16_i16_i16_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i16_i16_i16_i16_float {
    pub d0: i16,
    pub d1: i16,
    pub d2: i16,
    pub d3: i16,
}

impl i16_i16_i16_i16_float {
    pub fn new(d0: i16, d1: i16, d2: i16, d3: i16) -> i16_i16_i16_i16_float {
        i16_i16_i16_i16_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i16, i16, i16, i16)> for i16_i16_i16_i16_float {
    fn from(other: (i16, i16, i16, i16)) -> Self {
        i16_i16_i16_i16_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_ {
    pub d0: i32,
}

impl i32_ {
    pub fn new(d0: i32) -> i32_ {
        i32_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<i32> for i32_ {
    fn from(other: i32) -> Self {
        i32_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32 {
    pub d0: i32,
    pub d1: i32,
}

impl i32_i32 {
    pub fn new(d0: i32, d1: i32) -> i32_i32 {
        i32_i32 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32)> for i32_i32 {
    fn from(other: (i32, i32)) -> Self {
        i32_i32::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32_i32 {
    pub d0: i32,
    pub d1: i32,
    pub d2: i32,
}

impl i32_i32_i32 {
    pub fn new(d0: i32, d1: i32, d2: i32) -> i32_i32_i32 {
        i32_i32_i32 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32, i32)> for i32_i32_i32 {
    fn from(other: (i32, i32, i32)) -> Self {
        i32_i32_i32::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32_i32_i32 {
    pub d0: i32,
    pub d1: i32,
    pub d2: i32,
    pub d3: i32,
}

impl i32_i32_i32_i32 {
    pub fn new(d0: i32, d1: i32, d2: i32, d3: i32) -> i32_i32_i32_i32 {
        i32_i32_i32_i32 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32, i32, i32)> for i32_i32_i32_i32 {
    fn from(other: (i32, i32, i32, i32)) -> Self {
        i32_i32_i32_i32::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_float {
    pub d0: i32,
}

impl i32_float {
    pub fn new(d0: i32) -> i32_float {
        i32_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<i32> for i32_float {
    fn from(other: i32) -> Self {
        i32_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32_float {
    pub d0: i32,
    pub d1: i32,
}

impl i32_i32_float {
    pub fn new(d0: i32, d1: i32) -> i32_i32_float {
        i32_i32_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32)> for i32_i32_float {
    fn from(other: (i32, i32)) -> Self {
        i32_i32_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32_i32_float {
    pub d0: i32,
    pub d1: i32,
    pub d2: i32,
}

impl i32_i32_i32_float {
    pub fn new(d0: i32, d1: i32, d2: i32) -> i32_i32_i32_float {
        i32_i32_i32_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32, i32)> for i32_i32_i32_float {
    fn from(other: (i32, i32, i32)) -> Self {
        i32_i32_i32_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i32_i32_i32_i32_float {
    pub d0: i32,
    pub d1: i32,
    pub d2: i32,
    pub d3: i32,
}

impl i32_i32_i32_i32_float {
    pub fn new(d0: i32, d1: i32, d2: i32, d3: i32) -> i32_i32_i32_i32_float {
        i32_i32_i32_i32_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(i32, i32, i32, i32)> for i32_i32_i32_i32_float {
    fn from(other: (i32, i32, i32, i32)) -> Self {
        i32_i32_i32_i32_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_ {
    pub d0: u8,
}

impl u8_ {
    pub fn new(d0: u8) -> u8_ {
        u8_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::UNSIGNED_BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<u8> for u8_ {
    fn from(other: u8) -> Self {
        u8_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8 {
    pub d0: u8,
    pub d1: u8,
}

impl u8_u8 {
    pub fn new(d0: u8, d1: u8) -> u8_u8 {
        u8_u8 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::UNSIGNED_BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8)> for u8_u8 {
    fn from(other: (u8, u8)) -> Self {
        u8_u8::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8_u8 {
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
}

impl u8_u8_u8 {
    pub fn new(d0: u8, d1: u8, d2: u8) -> u8_u8_u8 {
        u8_u8_u8 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::UNSIGNED_BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8, u8)> for u8_u8_u8 {
    fn from(other: (u8, u8, u8)) -> Self {
        u8_u8_u8::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8_u8_u8 {
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
    pub d3: u8,
}

impl u8_u8_u8_u8 {
    pub fn new(d0: u8, d1: u8, d2: u8, d3: u8) -> u8_u8_u8_u8 {
        u8_u8_u8_u8 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::UNSIGNED_BYTE, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8, u8, u8)> for u8_u8_u8_u8 {
    fn from(other: (u8, u8, u8, u8)) -> Self {
        u8_u8_u8_u8::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_float {
    pub d0: u8,
}

impl u8_float {
    pub fn new(d0: u8) -> u8_float {
        u8_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::UNSIGNED_BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<u8> for u8_float {
    fn from(other: u8) -> Self {
        u8_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8_float {
    pub d0: u8,
    pub d1: u8,
}

impl u8_u8_float {
    pub fn new(d0: u8, d1: u8) -> u8_u8_float {
        u8_u8_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::UNSIGNED_BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8)> for u8_u8_float {
    fn from(other: (u8, u8)) -> Self {
        u8_u8_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8_u8_float {
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
}

impl u8_u8_u8_float {
    pub fn new(d0: u8, d1: u8, d2: u8) -> u8_u8_u8_float {
        u8_u8_u8_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::UNSIGNED_BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8, u8)> for u8_u8_u8_float {
    fn from(other: (u8, u8, u8)) -> Self {
        u8_u8_u8_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u8_u8_u8_u8_float {
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
    pub d3: u8,
}

impl u8_u8_u8_u8_float {
    pub fn new(d0: u8, d1: u8, d2: u8, d3: u8) -> u8_u8_u8_u8_float {
        u8_u8_u8_u8_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::UNSIGNED_BYTE, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u8, u8, u8, u8)> for u8_u8_u8_u8_float {
    fn from(other: (u8, u8, u8, u8)) -> Self {
        u8_u8_u8_u8_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_ {
    pub d0: u16,
}

impl u16_ {
    pub fn new(d0: u16) -> u16_ {
        u16_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::UNSIGNED_SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<u16> for u16_ {
    fn from(other: u16) -> Self {
        u16_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16 {
    pub d0: u16,
    pub d1: u16,
}

impl u16_u16 {
    pub fn new(d0: u16, d1: u16) -> u16_u16 {
        u16_u16 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::UNSIGNED_SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16)> for u16_u16 {
    fn from(other: (u16, u16)) -> Self {
        u16_u16::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16_u16 {
    pub d0: u16,
    pub d1: u16,
    pub d2: u16,
}

impl u16_u16_u16 {
    pub fn new(d0: u16, d1: u16, d2: u16) -> u16_u16_u16 {
        u16_u16_u16 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::UNSIGNED_SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16, u16)> for u16_u16_u16 {
    fn from(other: (u16, u16, u16)) -> Self {
        u16_u16_u16::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16_u16_u16 {
    pub d0: u16,
    pub d1: u16,
    pub d2: u16,
    pub d3: u16,
}

impl u16_u16_u16_u16 {
    pub fn new(d0: u16, d1: u16, d2: u16, d3: u16) -> u16_u16_u16_u16 {
        u16_u16_u16_u16 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::UNSIGNED_SHORT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16, u16, u16)> for u16_u16_u16_u16 {
    fn from(other: (u16, u16, u16, u16)) -> Self {
        u16_u16_u16_u16::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_float {
    pub d0: u16,
}

impl u16_float {
    pub fn new(d0: u16) -> u16_float {
        u16_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::UNSIGNED_SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<u16> for u16_float {
    fn from(other: u16) -> Self {
        u16_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16_float {
    pub d0: u16,
    pub d1: u16,
}

impl u16_u16_float {
    pub fn new(d0: u16, d1: u16) -> u16_u16_float {
        u16_u16_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::UNSIGNED_SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16)> for u16_u16_float {
    fn from(other: (u16, u16)) -> Self {
        u16_u16_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16_u16_float {
    pub d0: u16,
    pub d1: u16,
    pub d2: u16,
}

impl u16_u16_u16_float {
    pub fn new(d0: u16, d1: u16, d2: u16) -> u16_u16_u16_float {
        u16_u16_u16_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::UNSIGNED_SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16, u16)> for u16_u16_u16_float {
    fn from(other: (u16, u16, u16)) -> Self {
        u16_u16_u16_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u16_u16_u16_u16_float {
    pub d0: u16,
    pub d1: u16,
    pub d2: u16,
    pub d3: u16,
}

impl u16_u16_u16_u16_float {
    pub fn new(d0: u16, d1: u16, d2: u16, d3: u16) -> u16_u16_u16_u16_float {
        u16_u16_u16_u16_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::UNSIGNED_SHORT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u16, u16, u16, u16)> for u16_u16_u16_u16_float {
    fn from(other: (u16, u16, u16, u16)) -> Self {
        u16_u16_u16_u16_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_ {
    pub d0: u32,
}

impl u32_ {
    pub fn new(d0: u32) -> u32_ {
        u32_ { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                1, // the number of components per generic vertex attribute
                                gl::UNSIGNED_INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<u32> for u32_ {
    fn from(other: u32) -> Self {
        u32_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32 {
    pub d0: u32,
    pub d1: u32,
}

impl u32_u32 {
    pub fn new(d0: u32, d1: u32) -> u32_u32 {
        u32_u32 { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                2, // the number of components per generic vertex attribute
                                gl::UNSIGNED_INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32)> for u32_u32 {
    fn from(other: (u32, u32)) -> Self {
        u32_u32::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32_u32 {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
}

impl u32_u32_u32 {
    pub fn new(d0: u32, d1: u32, d2: u32) -> u32_u32_u32 {
        u32_u32_u32 { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                3, // the number of components per generic vertex attribute
                                gl::UNSIGNED_INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32, u32)> for u32_u32_u32 {
    fn from(other: (u32, u32, u32)) -> Self {
        u32_u32_u32::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32_u32_u32 {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
}

impl u32_u32_u32_u32 {
    pub fn new(d0: u32, d1: u32, d2: u32, d3: u32) -> u32_u32_u32_u32 {
        u32_u32_u32_u32 { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribIPointer(location as gl::types::GLuint,
                                4, // the number of components per generic vertex attribute
                                gl::UNSIGNED_INT, // data type
                                stride as gl::types::GLint,
                                offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32, u32, u32)> for u32_u32_u32_u32 {
    fn from(other: (u32, u32, u32, u32)) -> Self {
        u32_u32_u32_u32::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_float {
    pub d0: u32,
}

impl u32_float {
    pub fn new(d0: u32) -> u32_float {
        u32_float { d0 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               1, // the number of components per generic vertex attribute
                               gl::UNSIGNED_INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<u32> for u32_float {
    fn from(other: u32) -> Self {
        u32_float::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32_float {
    pub d0: u32,
    pub d1: u32,
}

impl u32_u32_float {
    pub fn new(d0: u32, d1: u32) -> u32_u32_float {
        u32_u32_float { d0, d1 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               2, // the number of components per generic vertex attribute
                               gl::UNSIGNED_INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32)> for u32_u32_float {
    fn from(other: (u32, u32)) -> Self {
        u32_u32_float::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32_u32_float {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
}

impl u32_u32_u32_float {
    pub fn new(d0: u32, d1: u32, d2: u32) -> u32_u32_u32_float {
        u32_u32_u32_float { d0, d1, d2 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               3, // the number of components per generic vertex attribute
                               gl::UNSIGNED_INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32, u32)> for u32_u32_u32_float {
    fn from(other: (u32, u32, u32)) -> Self {
        u32_u32_u32_float::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u32_u32_u32_u32_float {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
}

impl u32_u32_u32_u32_float {
    pub fn new(d0: u32, d1: u32, d2: u32, d3: u32) -> u32_u32_u32_u32_float {
        u32_u32_u32_u32_float { d0, d1, d2, d3 }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(location as gl::types::GLuint,
                               4, // the number of components per generic vertex attribute
                               gl::UNSIGNED_INT, // data type
                               gl::TRUE, // normalized (int-to-float conversion)
                               stride as gl::types::GLint,
                               offset as *const gl::types::GLvoid);
    }
}

impl From<(u32, u32, u32, u32)> for u32_u32_u32_u32_float {
    fn from(other: (u32, u32, u32, u32)) -> Self {
        u32_u32_u32_u32_float::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f16_ {
    pub d0: ::half::f16,
}

impl f16_ {
    pub fn new(d0: ::half::f16) -> f16_ {
        f16_ {
            d0
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            1, // the number of components per generic vertex attribute
            gl::HALF_FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<::half::f16> for f16_ {
    fn from(other: ::half::f16) -> Self {
        f16_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f16_f16 {
    pub d0: ::half::f16,
    pub d1: ::half::f16,
}

impl f16_f16 {
    pub fn new(d0: ::half::f16, d1: ::half::f16) -> f16_f16 {
        f16_f16 {
            d0, d1
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            2, // the number of components per generic vertex attribute
            gl::HALF_FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(::half::f16, ::half::f16)> for f16_f16 {
    fn from(other: (::half::f16, ::half::f16)) -> Self {
        f16_f16::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f16_f16_f16 {
    pub d0: ::half::f16,
    pub d1: ::half::f16,
    pub d2: ::half::f16,
}

impl f16_f16_f16 {
    pub fn new(d0: ::half::f16, d1: ::half::f16, d2: ::half::f16) -> f16_f16_f16 {
        f16_f16_f16 {
            d0, d1, d2
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::HALF_FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(::half::f16, ::half::f16, ::half::f16)> for f16_f16_f16 {
    fn from(other: (::half::f16, ::half::f16, ::half::f16)) -> Self {
        f16_f16_f16::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f16_f16_f16_f16 {
    pub d0: ::half::f16,
    pub d1: ::half::f16,
    pub d2: ::half::f16,
    pub d3: ::half::f16,
}

impl f16_f16_f16_f16 {
    pub fn new(d0: ::half::f16, d1: ::half::f16, d2: ::half::f16, d3: ::half::f16) -> f16_f16_f16_f16 {
        f16_f16_f16_f16 {
            d0, d1, d2, d3
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::HALF_FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(::half::f16, ::half::f16, ::half::f16, ::half::f16)> for f16_f16_f16_f16 {
    fn from(other: (::half::f16, ::half::f16, ::half::f16, ::half::f16)) -> Self {
        f16_f16_f16_f16::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f32_ {
    pub d0: f32,
}

impl f32_ {
    pub fn new(d0: f32) -> f32_ {
        f32_ {
            d0
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            1, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<f32> for f32_ {
    fn from(other: f32) -> Self {
        f32_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f32_f32 {
    pub d0: f32,
    pub d1: f32,
}

impl f32_f32 {
    pub fn new(d0: f32, d1: f32) -> f32_f32 {
        f32_f32 {
            d0, d1
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            2, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f32, f32)> for f32_f32 {
    fn from(other: (f32, f32)) -> Self {
        f32_f32::new(other.0, other.1)
    }
}

// -----------------------------------------

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

// -----------------------------------------

#[repr(C, packed)]
pub struct f32_f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
    pub d3: f32,
}

impl f32_f32_f32_f32 {
    pub fn new(d0: f32, d1: f32, d2: f32, d3: f32) -> f32_f32_f32_f32 {
        f32_f32_f32_f32 {
            d0, d1, d2, d3
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f32, f32, f32, f32)> for f32_f32_f32_f32 {
    fn from(other: (f32, f32, f32, f32)) -> Self {
        f32_f32_f32_f32::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f64_ {
    pub d0: f64,
}

impl f64_ {
    pub fn new(d0: f64) -> f64_ {
        f64_ {
            d0
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribLPointer(
            location as gl::types::GLuint,
            1, // the number of components per generic vertex attribute
            gl::DOUBLE, // data type
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<f64> for f64_ {
    fn from(other: f64) -> Self {
        f64_::new(other)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f64_f64 {
    pub d0: f64,
    pub d1: f64,
}

impl f64_f64 {
    pub fn new(d0: f64, d1: f64) -> f64_f64 {
        f64_f64 {
            d0, d1
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribLPointer(
            location as gl::types::GLuint,
            2, // the number of components per generic vertex attribute
            gl::DOUBLE, // data type
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f64, f64)> for f64_f64 {
    fn from(other: (f64, f64)) -> Self {
        f64_f64::new(other.0, other.1)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f64_f64_f64 {
    pub d0: f64,
    pub d1: f64,
    pub d2: f64,
}

impl f64_f64_f64 {
    pub fn new(d0: f64, d1: f64, d2: f64) -> f64_f64_f64 {
        f64_f64_f64 {
            d0, d1, d2
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribLPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::DOUBLE, // data type
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f64, f64, f64)> for f64_f64_f64 {
    fn from(other: (f64, f64, f64)) -> Self {
        f64_f64_f64::new(other.0, other.1, other.2)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct f64_f64_f64_f64 {
    pub d0: f64,
    pub d1: f64,
    pub d2: f64,
    pub d3: f64,
}

impl f64_f64_f64_f64 {
    pub fn new(d0: f64, d1: f64, d2: f64, d3: f64) -> f64_f64_f64_f64 {
        f64_f64_f64_f64 {
            d0, d1, d2, d3
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribLPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::DOUBLE, // data type
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<(f64, f64, f64, f64)> for f64_f64_f64_f64 {
    fn from(other: (f64, f64, f64, f64)) -> Self {
        f64_f64_f64_f64::new(other.0, other.1, other.2, other.3)
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i2_i10_i10_i10_rev {
    pub inner: u32, // TODO: nicer abstraction
}

impl i2_i10_i10_i10_rev {
    pub fn new(inner: u32) -> i2_i10_i10_i10_rev {
        i2_i10_i10_i10_rev {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::INT_2_10_10_10_REV, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u2_u10_u10_u10_rev {
    pub inner: ::vec_2_10_10_10::Vector,
}

impl u2_u10_u10_u10_rev {
    pub fn new(inner: ::vec_2_10_10_10::Vector) -> u2_u10_u10_u10_rev {
        u2_u10_u10_u10_rev {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::UNSIGNED_INT_2_10_10_10_REV, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<::vec_2_10_10_10::Vector> for u2_u10_u10_u10_rev {
    fn from(other: ::vec_2_10_10_10::Vector) -> Self {
        u2_u10_u10_u10_rev {
            inner: other
        }
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u10_u11_u11_rev {
    pub inner: u32, // TODO: nicer abstraction
}

impl u10_u11_u11_rev {
    pub fn new(inner: u32) -> u10_u11_u11_rev {
        u10_u11_u11_rev {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::UNSIGNED_INT_10F_11F_11F_REV, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct i2_i10_i10_i10_rev_float {
    pub inner: u32, // TODO: nicer abstraction
}

impl i2_i10_i10_i10_rev_float {
    pub fn new(inner: u32) -> i2_i10_i10_i10_rev_float {
        i2_i10_i10_i10_rev_float {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::INT_2_10_10_10_REV, // data type
            gl::TRUE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u2_u10_u10_u10_rev_float {
    pub inner: ::vec_2_10_10_10::Vector,
}

impl u2_u10_u10_u10_rev_float {
    pub fn new(inner: ::vec_2_10_10_10::Vector) -> u2_u10_u10_u10_rev_float {
        u2_u10_u10_u10_rev_float {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            4, // the number of components per generic vertex attribute
            gl::UNSIGNED_INT_2_10_10_10_REV, // data type
            gl::TRUE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}

impl From<::vec_2_10_10_10::Vector> for u2_u10_u10_u10_rev_float {
    fn from(other: ::vec_2_10_10_10::Vector) -> Self {
        u2_u10_u10_u10_rev_float {
            inner: other
        }
    }
}

// -----------------------------------------

#[repr(C, packed)]
pub struct u10_u11_u11_rev_float {
    pub inner: u32, // TODO: nicer abstraction
}

impl u10_u11_u11_rev_float {
    pub fn new(inner: u32) -> u10_u11_u11_rev_float {
        u10_u11_u11_rev_float {
            inner
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::UNSIGNED_INT_10F_11F_11F_REV, // data type
            gl::TRUE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid
        );
    }
}