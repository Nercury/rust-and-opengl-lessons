use gl;
use render::WindowSize;
use render_gl::Viewport;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::Window;

#[derive(PartialEq)]
pub enum HandleResult {
    Quit,
    Continue,
}

pub fn handle_default_window_events(
    event: &Event,
    gl: &gl::Gl,
    window: &Window,
    window_size: &mut WindowSize,
    viewport: &mut Viewport
) -> HandleResult {
    match event {
        Event::Quit { .. } => return HandleResult::Quit,
        Event::Window {
            win_event: WindowEvent::Resized(w, h),
            ..
        } => {
            if window_size.high_dpi {
                let (hdpi_w, hdpi_h) = window.drawable_size();
                println!("drawable_size: {:?}", (hdpi_w, hdpi_h));

                viewport.update_size(hdpi_w as i32, hdpi_h as i32);
                viewport.set_used(&gl);
                *window_size = WindowSize {
                    width: *w as i32,
                    height: *h as i32,
                    highdpi_width: hdpi_w as i32,
                    highdpi_height: hdpi_h as i32,
                    high_dpi: window_size.high_dpi,
                };
            } else {
                viewport.update_size(*w as i32, *h as i32);
                viewport.set_used(&gl);
                *window_size = WindowSize {
                    width: *w as i32,
                    height: *h as i32,
                    highdpi_width: *w as i32,
                    highdpi_height: *h as i32,
                    high_dpi: window_size.high_dpi,
                };
            }
            //camera.update_aspect(hdpi_w as f32 / hdpi_h as f32);
        }
        _ => (),
    };

    HandleResult::Continue
}
