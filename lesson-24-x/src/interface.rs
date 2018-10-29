use ui::*;
use ui::controls;
use gl;
use na;
use render_gl::ColorBuffer;
use render_gl::{DebugLines, RectMarker};
use std::collections::HashMap;

struct Location {
    size: (i32, i32),
    pos: (i32, i32),
}

struct ControlInfo {
    loc: Option<Location>,
    marker: Option<RectMarker>,
}

pub struct Interface {
    fill: Leaf<controls::Fill>,
    events: Events,
//    button: Leaf<controls::Button>,
    controls: HashMap<Ix, ControlInfo>,
    event_read_buffer: Vec<Effect>,
}

impl Interface {
    pub fn new(size: ElementSize) -> Interface {
        let tree = Tree::new();

        let events = tree.events();
        let fill = tree.create_root(controls::Fill::new());

//        let button = fill.add(controls::Button::new());

        fill.resize(size);

        Interface {
            fill,
            events,
//            button,
            controls: HashMap::new(),
            event_read_buffer: Vec::new(),
        }
    }

    pub fn resize(&mut self, size: ElementSize) {
        self.fill.resize(size);
    }

    fn process_events(&mut self, debuglines: &DebugLines) {
        self.events.drain_into(&mut self.event_read_buffer);

        for event in self.event_read_buffer.drain(..) {
            match event {
                Effect::Add { id, size } => {
                    self.controls.insert(id, ControlInfo {
                        loc: size.map(|xy| Location { size: xy, pos: (0, 0) }),
                        marker: None,
                    });
                },
                _ => ()
            }
        }
    }

    pub fn update(&mut self, _delta: f32, debuglines: &DebugLines) {
        self.process_events(debuglines)
    }

    pub fn mouse_move(&mut self, _x: i32, _y: i32) {

    }

    pub fn mouse_down(&mut self, _x: i32, _y: i32) {

    }

    pub fn mouse_up(&mut self, _x: i32, _y: i32) {

    }

    pub fn render(&mut self, _gl: &gl::Gl, _target: &ColorBuffer, _vp_matrix: &na::Matrix4<f32>) {

    }
}