use camera::TargetCamera;
use nalgebra as na;
use ncollide3d;
use render::WindowSize;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use selection::Selectables;

pub struct SelectablesInput {
    previous_device_ray: Option<na::Vector3<f32>>,
}

impl SelectablesInput {
    pub fn new() -> SelectablesInput {
        SelectablesInput {
            previous_device_ray: None,
        }
    }

    pub fn handle_selectable_events(
        &mut self,
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

                let device_ray_pos = (camera.get_inverse_p_matrix() * device_cursor)
                    .fixed_resize::<na::U2, na::U1>(0.0);
                let device_ray = inverse_view_matrix
                    * na::Vector4::new(device_ray_pos.x, device_ray_pos.y, -1.0, 0.0);
                let device_ray = device_ray.fixed_resize::<na::U3, na::U1>(0.0).normalize();
                self.previous_device_ray = Some(device_ray);
                Self::cast_ray_for_camera(&device_ray, camera, selectables);
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

    fn cast_ray_for_camera(
        device_ray: &na::Vector3<f32>,
        camera: &TargetCamera,
        selectables: &Selectables,
    ) {
        let ray = ncollide3d::query::Ray::new(camera.project_pos(), *device_ray);
        selectables.cast_cursor(&ray, &camera.target, &camera.direction());
    }

    pub fn update(&mut self, camera: &TargetCamera, selectables: &Selectables) {
        if let Some(ref device_ray) = self.previous_device_ray {
            Self::cast_ray_for_camera(device_ray, camera, selectables);
        }
    }
}
