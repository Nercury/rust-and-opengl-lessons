extern crate gl;
extern crate resources;
#[macro_use] extern crate failure;

pub mod data;
mod shader;

pub use self::shader::{Shader, Program, Error};