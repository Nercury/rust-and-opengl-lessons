extern crate sdl2;
extern crate gl;
extern crate vec_2_10_10_10;
extern crate half;
extern crate slab;
extern crate nalgebra;
extern crate ncollide3d;
extern crate image;
extern crate floating_duration;
extern crate tobj;
extern crate once_cell;
#[macro_use] extern crate failure;
#[macro_use] extern crate lesson_23_x_render_gl_derive as render_gl_derive;

pub mod camera;
pub mod render_gl;
pub mod render;
pub mod resources;
pub mod mesh;
pub mod selection;
pub mod dices;
pub mod propagation;
pub mod system;
mod debug;

use failure::err_msg;
use resources::Resources;
use nalgebra as na;
use std::time::{Instant, Duration};
use floating_duration::TimeAsFloat;
use system::profiling::alloc_watch::PeekAlloc;
use system::profiling::gl_watch;

#[global_allocator]
static GLOBAL: PeekAlloc = PeekAlloc;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    PeekAlloc::init();

    let res = Resources::from_relative_exe_path("assets-23-x").unwrap();

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
        .window("Game", window_size.width as u32, window_size.height as u32)
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

    let mut frame_profiler = render_gl::FrameProfiler::new(&gl, &res, 80)?;
    let mut allocation_profiler = render_gl::EventCountProfiler::new(&gl, &res, 3, 0)?;
    let mut gl_call_profiler = render_gl::EventCountProfiler::new(&gl, &res, 1, 20)?;

    let mut viewport = render_gl::Viewport::for_window(window_size.highdpi_width, window_size.highdpi_height);
    let color_buffer = render_gl::ColorBuffer::new();
    let mut editor_lines = render_gl::DebugLines::new(&gl, &res)?;
    let mut debug_lines = render_gl::DebugLines::new(&gl, &res)?;
    let _grid = editor_lines.grid_marker(na::Isometry3::identity(), 1.0, 100, [0.5, 0.5, 0.5, 1.0].into());
    let selectables = selection::Selectables::new();
    let mut render_selectables = system::render::selectables::RenderSelectables::new();
    let mut input_selectables = system::input::selectables::SelectablesInput::new();

    let mut dices = Vec::new();
    for x in -3..=3 {
        for y in -3..=3 {
            let mut dice = dices::Dice::new(&res, &gl, &debug_lines, &selectables)?;
            dice.set_transform(na::Isometry3::from_parts(na::Translation3::from_vector(
                [4.0 * x as f32, 4.0 * y as f32, 0.0].into()
            ), na::UnitQuaternion::identity()));
            dices.push(dice);
        }
    }

    let mut camera = camera::TargetCamera::new(
        window_size.width as f32 / window_size.height as f32,
        3.14 / 2.5,
        0.01,
        1000.0,
        3.14 / 4.0,
        5.0
    );
    let camera_target_marker = editor_lines.marker(camera.target, 0.25);

    // set up shared state for window

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(0.3, 0.3, 0.5));
    let mut side_cam = false;

    // main loop

    let mut time = Instant::now();

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        PeekAlloc::reset();
        gl_watch::reset();

        frame_profiler.begin();
        allocation_profiler.begin();
        gl_call_profiler.begin();

        for event in event_pump.poll_iter() {
            if system::input::window::handle_default_window_events(&event, &gl, &window, &mut window_size, &mut viewport, &mut camera) == system::input::window::HandleResult::Quit {
                break 'main;
            }
            system::input::camera::handle_camera_events(&event, &mut camera);
            input_selectables.handle_selectable_events(&event, &window_size, &camera, &selectables);

            match event {
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::C), .. } => {
                    side_cam = !side_cam;
                },
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::I), .. } => {
                    debug_lines.toggle();
                },
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), .. } => {
                    frame_profiler.toggle();
                    allocation_profiler.toggle();
                    gl_call_profiler.toggle();
                },
                _ => (),
            }
        }

        frame_profiler.push(render::color_white());

        let delta = time.elapsed().as_fractional_secs() as f32;
        time = Instant::now();
        if camera.update(delta) {
            camera_target_marker.update_position(camera.target);
        }
        input_selectables.update(&camera, &selectables);
        for dice in &mut dices {
            dice.update(delta);
        }
        render_selectables.update(&selectables, &editor_lines);

        frame_profiler.push(render::color_yellow());

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }
        let vp_matrix = if side_cam {
            camera.get_p_matrix() * na::Matrix4::look_at_rh(&na::Point3::new(-2.0, -2.0, 2.0), &na::Point3::origin(), &na::Vector3::z_axis())
        } else {
            camera.get_vp_matrix()
        };

        color_buffer.clear(&gl);

        frame_profiler.push(render::color_white());

        for dice in &mut dices {
            dice.render(&gl, &vp_matrix, &camera.project_pos().coords);
        }

        frame_profiler.push(render::color_red());

        debug_lines.render(&gl, &color_buffer, &vp_matrix);

        frame_profiler.push(render::color_white());

        editor_lines.render(&gl, &color_buffer, &vp_matrix);

        frame_profiler.push(render::color_gray());

        let left = 0;
        let top = window_size.highdpi_height;
        let right = window_size.highdpi_width;
        let bottom = 0;

        let ui_matrix = na::Matrix4::new_orthographic(left as f32, right as f32, bottom as f32, top as f32, -10.0, 10.0);

        frame_profiler.render(&gl, &color_buffer,&ui_matrix,
                        window_size.highdpi_width, window_size.highdpi_height);
        allocation_profiler.render(&gl, &color_buffer,&ui_matrix, window_size.highdpi_width);
        gl_call_profiler.render(&gl, &color_buffer,&ui_matrix, window_size.highdpi_width);

        frame_profiler.push(render::color_green());

        while time.elapsed() < Duration::from_millis(12) {
            ::std::thread::yield_now()
        }

        if let Some(values) = PeekAlloc::peek() {
            if values.alloc_num > 0 {
                allocation_profiler.push(values.alloc_num, render::color_white());
            }
            if values.dealloc_num > 0 {
                allocation_profiler.push(values.dealloc_num, render::color_black());
            }
        }

        let gl_errors = gl_watch::errors();
        if gl_errors > 0 {
            gl_call_profiler.push(gl_errors, render::color_red());
        }

        let gl_calls = gl_watch::calls();
        if gl_calls > 0 {
            gl_call_profiler.push(gl_calls, render::color_light_blue());
        }

        window.gl_swap_window();
    }

    Ok(())
}