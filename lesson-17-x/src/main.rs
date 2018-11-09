extern crate floating_duration;
extern crate gl;
extern crate half;
extern crate image;
extern crate nalgebra;
extern crate sdl2;
extern crate vec_2_10_10_10;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lesson_17_x_render_gl_derive as render_gl_derive;

pub mod camera;
mod cube;
mod debug;
pub mod render_gl;
pub mod resources;

use failure::err_msg;
use floating_duration::TimeAsFloat;
use nalgebra as na;
use resources::Resources;
use std::path::Path;
use std::time::Instant;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets-17-x")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let initial_window_size: (i32, i32) = (900, 700);

    let window = video_subsystem
        .window(
            "Game",
            initial_window_size.0 as u32,
            initial_window_size.1 as u32,
        ).opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    let mut viewport =
        render_gl::Viewport::for_window(initial_window_size.0, initial_window_size.1);
    let color_buffer = render_gl::ColorBuffer::new();
    let mut debug_lines = render_gl::DebugLines::new(&gl, &res)?;
    let cube = cube::Cube::new(&res, &gl, &debug_lines)?;

    let mut camera = camera::TargetCamera::new(
        initial_window_size.0 as f32 / initial_window_size.1 as f32,
        3.14 / 2.0,
        0.01,
        1000.0,
        3.14 / 4.0,
        3.0,
    );
    let camera_target_marker = debug_lines.marker(camera.target, 0.25);

    // set up shared state for window

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(0.3, 0.3, 0.5));

    // main loop

    let mut time = Instant::now();
    let mut side_cam = false;

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    scancode: Some(sdl2::keyboard::Scancode::C),
                    ..
                } => {
                    side_cam = !side_cam;
                }
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.set_used(&gl);
                    camera.update_aspect(w as f32 / h as f32);
                }
                e => handle_camera_event(&mut camera, &e),
            }
        }

        let delta = time.elapsed().as_fractional_secs();
        time = Instant::now();
        if camera.update(delta as f32) {
            camera_target_marker.update_position(camera.target);
        }

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }
        let vp_matrix = camera.get_vp_matrix();

        color_buffer.clear(&gl);
        cube.render(
            &gl,
            &camera.get_view_matrix(),
            &camera.get_p_matrix(),
            &camera.project_pos().coords,
        );
        debug_lines.render(&gl, &color_buffer, &vp_matrix);

        window.gl_swap_window();
    }

    Ok(())
}

fn handle_camera_event(camera: &mut camera::TargetCamera, e: &sdl2::event::Event) {
    use sdl2::event::Event;
    use sdl2::keyboard::Scancode;

    match *e {
        Event::MouseWheel { y, .. } => {
            camera.zoom(y as f32);
        }
        Event::KeyDown {
            scancode: Some(scancode),
            ..
        } => match scancode {
            Scancode::LShift | Scancode::RShift => camera.movement.faster = true,
            Scancode::A => camera.movement.left = true,
            Scancode::W => camera.movement.forward = true,
            Scancode::S => camera.movement.backward = true,
            Scancode::D => camera.movement.right = true,
            Scancode::Space => camera.movement.up = true,
            Scancode::LCtrl => camera.movement.down = true,
            _ => (),
        },
        Event::KeyUp {
            scancode: Some(scancode),
            ..
        } => match scancode {
            Scancode::LShift | Scancode::RShift => camera.movement.faster = false,
            Scancode::A => camera.movement.left = false,
            Scancode::W => camera.movement.forward = false,
            Scancode::S => camera.movement.backward = false,
            Scancode::D => camera.movement.right = false,
            Scancode::Space => camera.movement.up = false,
            Scancode::LCtrl => camera.movement.down = false,
            _ => (),
        },
        Event::MouseMotion {
            xrel,
            yrel,
            mousestate,
            ..
        } => {
            if mousestate.right() {
                camera.rotate(&na::Vector2::new(xrel as f32, -yrel as f32));
            }
        }
        _ => (),
    }
}
