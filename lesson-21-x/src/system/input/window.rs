use camera::TargetCamera;
use gl;
use render::WindowSize;
use render_gl::Viewport;
use sdl2::event::{Event, WindowEvent};

#[derive(PartialEq)]
pub enum HandleResult {
    Quit,
    Continue,
}

pub fn handle_default_window_events(
    event: &Event,
    gl: &gl::Gl,
    window_size: &mut WindowSize,
    viewport: &mut Viewport,
    camera: &mut TargetCamera,
) -> HandleResult {
    match event {
        Event::Quit { .. } => return HandleResult::Quit,
        Event::Window {
            win_event: WindowEvent::Resized(w, h),
            ..
        } => {
            viewport.update_size(*w as i32, *h as i32);
            viewport.set_used(&gl);
            *window_size = WindowSize {
                width: *w as i32,
                height: *h as i32,
            };
            camera.update_aspect(*w as f32 / *h as f32);
        }
        _ => (),
    };

    HandleResult::Continue
}
