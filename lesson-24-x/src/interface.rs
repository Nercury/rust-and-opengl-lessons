use ui::*;
use ui::controls;
use gl;
use na;
use render_gl::ColorBuffer;

pub struct Interface {
    c: Fill,
    button: Leaf<controls::Button>,
}

impl Interface {
    pub fn new() -> Interface {
        let mut c = Fill::root();

        let button = c.add(controls::Button::new());

        Interface {
            c,
            button,
        }
    }

    pub fn resize(&mut self, size: ElementSize) {
        self.c.resize(size);
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {

    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {

    }

    pub fn mouse_up(&mut self, x: i32, y: i32) {

    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {

    }
}