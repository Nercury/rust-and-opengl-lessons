extern crate sdl2;
extern crate gl;
extern crate vec_2_10_10_10;
extern crate half;
extern crate nalgebra;
extern crate floating_duration;
#[macro_use] extern crate failure;
#[macro_use] extern crate lesson_15_x_render_gl_derive as render_gl_derive;

pub mod camera;
pub mod render_gl;
pub mod resources;
mod debug;
mod triangle;

use failure::err_msg;
use resources::Resources;
use nalgebra as na;
use std::time::Instant;
use floating_duration::TimeAsFloat;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_exe_path()?;

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let initial_window_size: (i32, i32) = (900, 700);

    let window = video_subsystem
        .window("Game", initial_window_size.0 as u32, initial_window_size.1 as u32)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut viewport = render_gl::Viewport::for_window(initial_window_size.0, initial_window_size.1);
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
    let mut _p2 = Some(debug_lines
        .start_polyline(
            [0.5, 0.0, -0.5].into(),
            [1.0, 1.0, 0.0, 1.0].into()
        )
        .with_point(
            [0.0, 0.0, 0.5].into(),
            [1.0, 0.0, 0.0, 1.0].into(),
        )
        .with_point(
            [-0.5, 0.0, -0.5].into(),
            [1.0, 1.0, 0.0, 1.0].into(),
        )
        .close_and_finish());
    let triangle = triangle::Triangle::new(&res, &gl)?;

    let mut camera = camera::TargetCamera::new(
        initial_window_size.0 as f32 / initial_window_size.1 as f32,
        3.14 / 2.0,
        0.01,
        1000.0,
        3.14 / 4.0,
        2.0
    );
    let camera_target_marker = debug_lines.marker(camera.target, 0.25);
    let camera_position_marker = debug_lines.colored_marker(camera.project_pos(), [0.0, 1.0, 1.0, 1.0].into(),0.25);

    let mut camera_target_markers = (0..2).map(|i| debug_lines.marker(camera.target + na::Vector3::new(i as f32 + 1.0, 0.0, 0.0), 0.5)).collect::<Vec<_>>();
    let camera_pos = camera.project_pos();
    let camera_ray = debug_lines.ray_marker(camera_pos, camera.target - camera_pos, [1.0, 0.0, 0.0, 1.0].into());

//    let camera_direction_marker = debug_lines.colored_marker(camera.target + camera.get_direction(), [1.0, 1.0, 1.0, 1.0].into(), 0.5);

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
                sdl2::event::Event::Quit {..} => break 'main,
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::PageUp), .. } => {
                    let len = camera_target_markers.len();
                    camera_target_markers.push(debug_lines.marker(camera.target + na::Vector3::new(len as f32 + 1.0, 0.0, 0.0), 0.5));
                }
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::PageDown), .. } => {
                    camera_target_markers.pop();
                },
                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::C), .. } => {
                    side_cam = !side_cam;
                },
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.set_used(&gl);
                    camera.update_aspect(w as f32 / h as f32);
                },
                e => handle_camera_event(&mut camera, &e),
            }
        }

        let delta = time.elapsed().as_fractional_secs();
        time = Instant::now();
        if camera.update(delta as f32) {
            camera_target_marker.update_position(camera.target);
            camera_position_marker.update_position(camera.project_pos());
            for (i, m) in camera_target_markers.iter().enumerate() {
                m.update_position(camera.target + na::Vector3::new(i as f32 + 1.0, 0.0, 0.0));
            }
            let camera_pos = camera.project_pos();
            camera_ray.update_ray(camera_pos, camera.target - camera_pos);
        }
//        camera_direction_marker.update_position(camera.target + camera.get_direction());

        let vp_matrix = if side_cam {
            camera.get_p_matrix() * na::Matrix4::look_at_rh(&na::Point3::new(-2.0, -2.0, 2.0), &na::Point3::origin(), &na::Vector3::z_axis())
        } else {
            camera.get_vp_matrix()
        };

        unsafe {
            gl.Enable(gl::CULL_FACE);
        }

        color_buffer.clear(&gl);
        triangle.render(&gl, &vp_matrix);
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
        },
        Event::KeyDown { scancode: Some(scancode), .. } => {
            match scancode {
                Scancode::LShift | Scancode::RShift => camera.movement.faster = true,
                Scancode::A => camera.movement.left = true,
                Scancode::W => camera.movement.forward = true,
                Scancode::S => camera.movement.backward = true,
                Scancode::D => camera.movement.right = true,
                Scancode::Space => camera.movement.up = true,
                Scancode::LCtrl => camera.movement.down = true,
                _ => (),
            }
        }
        Event::KeyUp { scancode: Some(scancode), .. } => {
            match scancode {
                Scancode::LShift | Scancode::RShift => camera.movement.faster = false,
                Scancode::A => camera.movement.left = false,
                Scancode::W => camera.movement.forward = false,
                Scancode::S => camera.movement.backward = false,
                Scancode::D => camera.movement.right = false,
                Scancode::Space => camera.movement.up = false,
                Scancode::LCtrl => camera.movement.down = false,
                _ => (),
            }
        }
        Event::MouseMotion { xrel, yrel, mousestate, .. } => {
            if mousestate.right() {
                camera.rotate(&na::Vector2::new(xrel as f32, -yrel as f32));
            }
        },
        _ => (),
    }
}