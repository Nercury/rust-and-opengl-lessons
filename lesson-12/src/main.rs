extern crate sdl2;
extern crate gl;
extern crate vec_2_10_10_10;
extern crate half;
#[macro_use] extern crate failure;
#[macro_use] extern crate lesson_12_render_gl_derive as render_gl_derive;

pub mod render_gl;
pub mod resources;

use render_gl::data;
use render_gl::buffer;
use failure::err_msg;
use resources::Resources;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::u2_u10_u10_u10_rev_float,
}

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_exe_path()?;

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 2);

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // set up shader program

    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle")?;

    // set up vertex buffer object

    let vertices: Vec<Vertex> = vec![
        Vertex {
            pos: (0.5, -0.5, 0.0).into(),
            clr: (1.0, 0.0, 0.0, 1.0).into()
        }, // bottom right
        Vertex {
            pos: (-0.5, -0.5, 0.0).into(),
            clr: (0.0, 1.0, 0.0, 1.0).into()
        }, // bottom left
        Vertex {
            pos: (0.0,  0.5, 0.0).into(),
            clr: (0.0, 0.0, 1.0, 1.0).into()
        }  // top
    ];

    let vbo = buffer::ArrayBuffer::new(&gl);
    vbo.bind();
    vbo.static_draw_data(&vertices);
    vbo.unbind();

    // set up vertex array object

    let vao = buffer::VertexArray::new(&gl);

    vao.bind();
    vbo.bind();
    Vertex::vertex_attrib_pointers(&gl);
    vbo.unbind();
    vao.unbind();

    // set up shared state for window

    unsafe {
        gl.Viewport(0, 0, 900, 700);
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    // main loop

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        // draw triangle

        shader_program.set_used();
        vao.bind();
        unsafe {
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0, // starting index in the enabled arrays
                3 // number of indices to be rendered
            );
        }

        window.gl_swap_window();
    }

    Ok(())
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}