extern crate env_logger;
extern crate failure;
extern crate floating_duration;
extern crate gl;
extern crate lesson_24_x_render as render;
extern crate lesson_24_x_render_gl as render_gl;
#[macro_use] extern crate log;
extern crate nalgebra as na;
extern crate nalgebra_glm as glm;
extern crate resources;
extern crate sdl2;
extern crate ui;
extern crate lyon_path;
extern crate lyon_tessellation;
extern crate int_hash;

pub mod camera;
pub mod debug;
pub mod interface;
pub mod profiling;
pub mod system;

use failure::err_msg;
use floating_duration::TimeAsFloat;
use interface::Interface;
use std::time::{Duration, Instant};
use profiling::alloc;
use profiling::gl_calls;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, log::LevelFilter::Trace);
    builder.default_format_module_path(true);
    builder.default_format_level(true);
    if ::std::env::var("RUST_LOG").is_ok() {
        builder.parse(&::std::env::var("RUST_LOG").unwrap());
    }
    builder.init();

    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = resources::Resources::new().loaded_from(
        "core",
        0,
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
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);

    let mut window_size = render::WindowSize {
        width: 1800,
        height: 960,
        highdpi_width: 1800,
        highdpi_height: 960,
        high_dpi: true,
    };

    let mut window = video_subsystem
        .window("Demo", window_size.width as u32, window_size.height as u32);
    let builder = window
        .opengl()
        .resizable();

    if window_size.high_dpi {
        builder.allow_highdpi();
    }

    let window = builder.build()?;

    if window_size.high_dpi {
        let drawable_size = window.drawable_size();
        window_size.highdpi_width = drawable_size.0 as i32;
        window_size.highdpi_height = drawable_size.1 as i32;
    } else {
        window_size.highdpi_width = window_size.width;
        window_size.highdpi_height = window_size.height;
    }

    let mut scale = window_size.highdpi_width as f32 / window_size.width as f32;
    let mut scale_modifier = 1.0;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    // 0 for immediate updates,
    // 1 for updates synchronized with the vertical retrace,
    // -1 for late swap tearing

    let vsync = true;
    video_subsystem.gl_set_swap_interval(if vsync { 1 } else { 0 });

    let mut frame_profiler = render_gl::FrameProfiler::new(&gl, &resources, 80)?;
    frame_profiler.toggle();
    let mut allocation_profiler = render_gl::EventCountProfiler::new(&gl, &resources, 3, 0)?;
    allocation_profiler.toggle();
    let mut gl_call_profiler = render_gl::EventCountProfiler::new(&gl, &resources, 1, 20)?;
    gl_call_profiler.toggle();

    let mut viewport =
        render_gl::Viewport::for_window(window_size.highdpi_width, window_size.highdpi_height);
    let color_buffer = render_gl::ColorBuffer::new();

    // set up shared state for window

    let mut camera = camera::TargetCamera::new(
        window_size.width as f32 / window_size.height as f32,
        3.14 / 2.5,
        0.1,
        10000.0,
        3.14 / 4.0,
        500.0,
        na::Point3::new(window_size.highdpi_width as f32 / 2.0, -window_size.highdpi_height as f32 / 2.0, 0.0)
    );

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(1.0, 1.0, 1.0));
    color_buffer.enable_multisample(&gl);

    let mut iface_auto_size = false;
    let mut iface = Interface::new(
        &gl,
        &resources,
        ui::BoxSize::Fixed {
            w: viewport.w,
            h: viewport.h,
        },
        scale * scale_modifier
    )?;
//    iface.toggle_bounds();

    let mut perspective_view = false;

    // main loop

    let mut time = Instant::now();

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        alloc::reset();
        gl_calls::reset();

        frame_profiler.begin();
        allocation_profiler.begin();
        gl_call_profiler.begin();

        for event in event_pump.poll_iter() {
            if system::input::window::handle_default_window_events(
                &event,
                &gl,
                &window,
                &mut window_size,
                &mut viewport,
                &mut camera
            ) == system::input::window::HandleResult::Quit
            {
                break 'main;
            }
            system::input::camera::handle_camera_events(&event, &mut camera);

            use sdl2::event::Event;
            use sdl2::keyboard::Scancode;

            let iface_resize = match event {
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(_w, _h),
                    ..
                } => {
                    scale = window_size.highdpi_width as f32 / window_size.width as f32;
                    true
                }
                Event::KeyDown {
                    scancode: Some(Scancode::L),
                    ..
                } => {
                    iface_auto_size = !iface_auto_size;
                    true
                }
                Event::KeyDown {
                    scancode: Some(Scancode::T),
                    ..
                } => {
                    iface.toggle_wireframe();
                    false
                }
                Event::KeyDown {
                    scancode: Some(Scancode::C),
                    ..
                } => {
                    perspective_view = !perspective_view;
                    false
                }
                Event::KeyDown {
                    scancode: Some(Scancode::B),
                    ..
                } => {
                    iface.toggle_bounds();
                    false
                }
                Event::KeyDown {
                    scancode: Some(Scancode::LeftBracket),
                    ..
                } => {
                    scale_modifier /= 1.5;
                    true
                }
                Event::KeyDown {
                    scancode: Some(Scancode::RightBracket),
                    ..
                } => {
                    scale_modifier *= 1.5;
                    true
                }
                Event::KeyDown {
                    scancode: Some(Scancode::Left),
                    ..
                } => {
                    iface.send_action(ui::UiAction::PreviousSlide);
                    false
                }
                Event::KeyDown {
                    scancode: Some(Scancode::Right),
                    ..
                } => {
                    iface.send_action(ui::UiAction::NextSlide);
                    false
                }
                Event::KeyDown {
                    scancode: Some(Scancode::P),
                    ..
                } => {
                    frame_profiler.toggle();
                    allocation_profiler.toggle();
                    gl_call_profiler.toggle();
                    false
                }
                _ => false,
            };

            if iface_resize {
                println!("scale: {}", scale * scale_modifier);

                if iface_auto_size {
                    iface.resize(ui::BoxSize::Auto, scale * scale_modifier);
                } else {
                    iface.resize(ui::BoxSize::Fixed {
                        w: viewport.w,
                        h: viewport.h,
                    }, scale * scale_modifier);
                }
            }
        }

        frame_profiler.push(render::color_black());

        let delta = time.elapsed().as_fractional_secs() as f32;
        time = Instant::now();

        camera.update(delta);
        iface.update(delta);

        frame_profiler.push(render::color_blue());

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }

        color_buffer.clear(&gl);

        let ui_matrix = if !perspective_view {
            let left = 0;
            let top = 0;
            let right = window_size.highdpi_width;
            let bottom = -window_size.highdpi_height;

            na::Matrix4::new_orthographic(
                left as f32,
                right as f32,
                bottom as f32,
                top as f32,
                -10.0,
                10.0,
            )
        } else {
            camera.get_vp_matrix()
        };

        iface.render(&gl, &color_buffer, &ui_matrix);

        frame_profiler.push(render::color_red());

        let left = 0;
        let top = window_size.highdpi_height;
        let right = window_size.highdpi_width;
        let bottom = 0;

        let ui_matrix = na::Matrix4::new_orthographic(
            left as f32,
            right as f32,
            bottom as f32,
            top as f32,
            -10.0,
            10.0,
        );

        frame_profiler.render(
            &gl,
            &color_buffer,
            &ui_matrix,
            window_size.highdpi_width,
            window_size.highdpi_height,
        );
        allocation_profiler.render(&gl, &color_buffer, &ui_matrix, window_size.highdpi_width);
        gl_call_profiler.render(&gl, &color_buffer, &ui_matrix, window_size.highdpi_width);

        frame_profiler.push(render::color_green());

//        while time.elapsed() < Duration::from_millis(6) {
//            ::std::thread::yield_now()
//        }

        {
            let ac = alloc::alloc_count();
            let dc = alloc::dealloc_count();
            if ac > 0 {
                allocation_profiler.push(ac, render::color_blue());
            }
            if dc > 0 {
                allocation_profiler.push(dc, render::color_green());
            }
        }

        let gl_error_c = gl_calls::errors();
        if gl_error_c > 0 {
            gl_call_profiler.push(gl_error_c, render::color_red());
        }

        let gl_call_c = gl_calls::calls();
        if gl_call_c > 0 {
            gl_call_profiler.push(gl_call_c, render::color_light_blue());
        }

        while time.elapsed() < Duration::from_millis(6) {
            ::std::thread::yield_now()
        }

        window.gl_swap_window();

        frame_profiler.push(render::color_orange());
    }

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
