use ui::*;
use ui::controls;
use gl;
use na;
use render_gl::ColorBuffer;

pub struct Interface {
    fill: Fill,
    events: Events,
    button: Leaf<controls::Button>,
}

impl Interface {
    pub fn new(size: ElementSize) -> Interface {
        let tree = Tree::new();

        let events = tree.events();
        let fill = tree.create_root_fill();

        let button = fill.add(controls::Button::new());

        fill.resize(size);

        Interface {
            fill,
            events,
            button,
        }
    }

    pub fn resize(&mut self, size: ElementSize) {
        self.fill.resize(size);
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