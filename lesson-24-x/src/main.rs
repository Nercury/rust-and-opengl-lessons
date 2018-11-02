extern crate gl;
extern crate sdl2;
extern crate ui;
extern crate nalgebra as na;
extern crate failure;
extern crate lesson_24_x_render as render;
extern crate resources;
extern crate lesson_24_x_render_gl as render_gl;
extern crate floating_duration;

pub mod profiling;
pub mod debug;
pub mod system;
pub mod interface;

use failure::err_msg;
use std::time::{Instant, Duration};
use floating_duration::TimeAsFloat;
use interface::Interface;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = resources::Resources::new()
        .loaded_from(
            "core", 0,
            resources::backend::FileSystem::from_rel_path(env!("CARGO_MANIFEST_DIR"), "core")
                .with_write()
                .with_watch(),
        );

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);
    gl_attr.set_accelerated_visual(true);
    gl_attr.set_double_buffer(true);

    let mut window_size = render::WindowSize {
        width: 960,
        height: 600,
        highdpi_width: 960,
        highdpi_height: 600
    };

    let window = video_subsystem
        .window("Demo", window_size.width as u32, window_size.height as u32)
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()?;

    let drawable_size = window.drawable_size();
    window_size.highdpi_width = drawable_size.0 as i32;
    window_size.highdpi_height = drawable_size.1 as i32;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // 0 for immediate updates,
    // 1 for updates synchronized with the vertical retrace,
    // -1 for late swap tearing

    let vsync = false;
    video_subsystem.gl_set_swap_interval(if vsync { 1 } else { 0 });

    let mut viewport = render_gl::Viewport::for_window(window_size.highdpi_width, window_size.highdpi_height);
    let color_buffer = render_gl::ColorBuffer::new();

    // set up shared state for window

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(0.3, 0.3, 0.5));

    let mut iface = Interface::new(&gl, &resources, ui::ElementSize::Fixed { w: viewport.w, h: viewport.h })?;

    // main loop

    let mut time = Instant::now();

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {

        for event in event_pump.poll_iter() {
            if system::input::window::handle_default_window_events(&event, &gl, &window, &mut window_size, &mut viewport) == system::input::window::HandleResult::Quit {
                break 'main;
            }

            match event {
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(_w, _h),
                    ..
                } => {
                    let (hdpi_w, hdpi_h) = window.drawable_size();
                    viewport.update_size(hdpi_w as i32, hdpi_h as i32);
                    viewport.set_used(&gl);
                    iface.resize(ui::ElementSize::Fixed { w: hdpi_w as i32, h: hdpi_h as i32 });
                },
                _ => (),
            };
        }

        let delta = time.elapsed().as_fractional_secs() as f32;
        time = Instant::now();

        iface.update(delta);

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }

        color_buffer.clear(&gl);

        let left = 0;
        let top = window_size.highdpi_height;
        let right = window_size.highdpi_width;
        let bottom = 0;

        let ui_matrix = na::Matrix4::new_nonuniform_scaling(&[1.0, -1.0, 1.0].into())
            * na::Matrix4::new_orthographic(left as f32, right as f32, bottom as f32, top as f32, -10.0, 10.0);

        iface.render(&gl, &color_buffer, &ui_matrix);

        while time.elapsed() < Duration::from_millis(12) {
            ::std::thread::yield_now()
        }

        window.gl_swap_window();
    }

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
