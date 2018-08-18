extern crate gl;

pub mod profiling;

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;

fn main () {
    println!("Hello");
}