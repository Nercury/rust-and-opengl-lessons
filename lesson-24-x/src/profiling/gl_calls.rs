#[cfg(feature = "gl_debug")]
pub fn reset() {
    use gl;
    gl::profiler_reset();
}

#[cfg(not(feature = "gl_debug"))]
pub fn reset() {}

#[cfg(feature = "gl_debug")]
pub fn calls() -> usize {
    use gl;
    gl::profiler_call_count()
}

#[cfg(not(feature = "gl_debug"))]
pub fn calls() -> usize {
    0
}

#[cfg(feature = "gl_debug")]
pub fn errors() -> usize {
    use gl;
    gl::profiler_err_count()
}

#[cfg(not(feature = "gl_debug"))]
pub fn errors() -> usize {
    0
}
