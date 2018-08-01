extern crate sdl2;
extern crate gl;
extern crate vec_2_10_10_10;
extern crate half;
extern crate nalgebra;
#[macro_use] extern crate failure;
#[macro_use] extern crate lesson_14_x_render_gl_derive as render_gl_derive;

pub mod render_gl;
pub mod resources;
mod debug;

use failure::err_msg;
use resources::Resources;
use std::path::Path;
use nalgebra as na;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets-14-x")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut viewport = render_gl::Viewport::for_window(900, 700);
    let color_buffer = render_gl::ColorBuffer::new();
    let mut debug_lines = render_gl::DebugLines::new(&gl, &res)?;
    let mut _p = Some(debug_lines
        .start_polyline(
            [0.5, -0.5, 0.0].into(),
            [1.0, 0.0, 0.0, 1.0].into()
        )
        .with_point(
            [0.0, 0.5, 0.0].into(),
            [0.0, 1.0, 0.0, 1.0].into(),
        )
        .with_point(
            [-0.5, -0.5, 0.0].into(),
            [1.0, 1.0, 0.0, 0.0].into(),
        )
        .close_and_finish());

    // set up shared state for window

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(0.3, 0.3, 0.5));

    // main loop

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.set_used(&gl);
                },
                _ => {},
            }
        }

        color_buffer.clear(&gl);
        debug_lines.render(&gl, &color_buffer);

        window.gl_swap_window();
    }

    Ok(())
}