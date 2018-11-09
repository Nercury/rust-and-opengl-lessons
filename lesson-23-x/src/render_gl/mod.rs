pub mod buffer;
pub mod data;

mod color_buffer;
mod debug_lines;
mod profiler;
mod shader;
mod texture;
mod viewport;

use gl;

pub use self::color_buffer::ColorBuffer;
pub use self::debug_lines::{AabbMarker, DebugLines, RayMarkers};
pub use self::profiler::{EventCountProfiler, FrameProfiler};
pub use self::shader::{Error, Program, Shader};
pub use self::texture::{Texture, TextureLoadBuilder, TextureLoadOptions};
pub use self::viewport::Viewport;

fn gl_error_to_str(error: u32) -> &'static str {
    match error {
        gl::NO_ERROR => {
            "NO_ERROR = No error has been recorded.
                        The value of this \
                      symbolic constant is guaranteed to be 0."
        }
        gl::INVALID_ENUM => {
            "INVALID_ENUM = An unacceptable value is specified for an enumerated argument.
                        \
                      The offending command is ignored
                        and has no other \
                      side effect than to set the error flag."
        }
        gl::INVALID_VALUE => {
            "INVALID_VALUE = A numeric argument is out of range.
                        The offending command is ignored
                        and has no other side effect than to set the error flag."
        }
        gl::INVALID_OPERATION => {
            "INVALID_OPERATION = The specified operation is not allowed in the current \
                      state.
                        The offending command is ignored
                        \
                      and has no other side effect than to set the error flag."
        }
        gl::INVALID_FRAMEBUFFER_OPERATION => {
            "INVALID_FRAMEBUFFER_OPERATION = The command is trying to render to or read \
                      from the framebuffer
                        while the currently bound \
                      framebuffer is not framebuffer
                        complete (i.e. the \
                      return value from
                        glCheckFramebufferStatus
                        \
                      is not GL_FRAMEBUFFER_COMPLETE).
                        The offending \
                      command is ignored
                        and has no other side effect than \
                      to set the error flag."
        }
        gl::OUT_OF_MEMORY => {
            "OUT_OF_MEMORY = There is not enough memory left to execute the command.
                        The state of the GL is undefined,
                        except for the state of the error flags,
                        after this error is recorded."
        }
        _ => "Unknown error",
    }
}

pub fn check_err(gl: &gl::Gl) -> bool {
    let res = unsafe { gl.GetError() };
    if res == gl::NO_ERROR {
        return false;
    }

    println!("GL error {}: {}", res, gl_error_to_str(res));
    true
}
