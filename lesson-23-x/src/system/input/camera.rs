use camera::TargetCamera;
use nalgebra as na;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub fn handle_camera_events(e: &Event, camera: &mut TargetCamera) {
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
