extern crate gl;
extern crate half;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate resources;
extern crate vec_2_10_10_10;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lesson_24_x_render_gl_derive as render_gl_derive;

mod debug_lines;
mod shader;

pub mod buffer;
pub mod color_buffer;
pub mod data;
pub mod viewport;

pub use self::color_buffer::ColorBuffer;
pub use self::debug_lines::{AabbMarker, DebugLines, RayMarkers, RectMarker};
pub use self::shader::{Error, Program, Shader};
pub use self::viewport::Viewport;
