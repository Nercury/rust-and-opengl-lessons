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
    parent_id: Option<Ix>,
    loc: Option<Location>,
    marker: Option<RectMarker>,
}

impl ControlInfo {
    pub fn new(parent_id: Option<Ix>, debug_lines: &DebugLines) -> ControlInfo {
        ControlInfo {
            parent_id,
            loc: None,
            marker: None,
        }
    }

    pub fn update(&mut self, debug_lines: &DebugLines, size: Option<(i32, i32)>) {
        self.loc = size.map(|wh| Location { size: wh, pos: (0, 0) });
        match (self.marker.is_some(), size) {
            (false, None) => (),
            (false, Some(wh)) => self.marker = Some(debug_lines.rect_marker(
                na::Isometry3::from_parts(na::Translation3::from_vector(
                    [0.5, 0.5, 0.0].into()
                ), na::UnitQuaternion::identity()),
                na::Vector2::new(wh.0 as f32, wh.1 as f32),
                na::Vector4::new(1.0, 0.5, 0.2, 1.0)
            )),
            (true, None) => self.marker = None,
            (true, Some(wh)) => self.marker.as_mut().unwrap().update_size_and_color(na::Vector2::new(wh.0 as f32, wh.1 as f32), na::Vector4::new(1.0, 0.5, 0.2, 1.0)),
        }
    }
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

    fn process_events(&mut self, debug_lines: &DebugLines) {
        self.events.drain_into(&mut self.event_read_buffer);

        for event in self.event_read_buffer.drain(..) {
            match event {
                Effect::Add { id, parent_id } => {
                    self.controls.insert(id, ControlInfo::new(parent_id, debug_lines));
                },
                Effect::Resize { id, size } => {
                    self.controls.get_mut(&id).map(|c| c.update(debug_lines, size));
                },
                Effect::Remove { id } => {
                    self.controls.remove(&id);
                },
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