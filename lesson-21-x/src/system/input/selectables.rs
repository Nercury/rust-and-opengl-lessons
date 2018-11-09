use camera::TargetCamera;
use nalgebra as na;
use ncollide3d;
use render::WindowSize;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use selection::Selectables;

pub fn handle_selectable_events(
    event: &Event,
    window_size: &WindowSize,
    camera: &TargetCamera,
    selectables: &Selectables,
) {
    match event {
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            ..
        } => {
            selectables.send_mouse_down();
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => {
            selectables.send_mouse_up();
        }
        Event::MouseMotion { x, y, .. } => {
            let device_cursor = na::Vector4::new(
                *x as f32 / window_size.width as f32 * 2.0 - 1.0,
                (1.0 - (*y as f32 / window_size.height as f32)) * 2.0 - 1.0,
                -1.0,
                1.0,
            );

            let inverse_view_matrix = camera.get_inverse_view_matrix();

            let device_ray_pos =
                (camera.get_inverse_p_matrix() * device_cursor).fixed_resize::<na::U2, na::U1>(0.0);
            let device_ray = inverse_view_matrix
                * na::Vector4::new(device_ray_pos.x, device_ray_pos.y, -1.0, 0.0);

            let ray = ncollide3d::query::Ray::new(
                camera.project_pos(),
                device_ray.fixed_resize::<na::U3, na::U1>(0.0).normalize(),
            );
            selectables.cast_cursor(&ray, &camera.direction());
        }
        Event::KeyDown {
            scancode: Some(Scancode::Escape),
            ..
        } => {
            selectables.cancel_drag();
        }
        _ => (),
    };
}
