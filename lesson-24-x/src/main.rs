extern crate gl;
extern crate failure;
extern crate resources;
extern crate lesson_24_x_render_gl;
#[macro_use]
extern crate lesson_24_x_render_gl_derive;

pub mod profiling;
pub mod debug;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = resources::Resources::new();

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
