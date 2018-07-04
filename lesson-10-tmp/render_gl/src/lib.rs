extern crate gl;
extern crate lesson_10_resources as resources;

#[macro_use]
extern crate failure;

#[allow(unused_imports)]
#[macro_use]
extern crate lesson_10_render_gl_derive as render_gl_derive;

pub use render_gl_derive::*;

pub mod data;
mod shader;

pub use self::shader::{Shader, Program, Error};