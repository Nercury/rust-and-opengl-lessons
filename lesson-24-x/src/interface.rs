use ui::*;
use ui::controls;
use gl;
use na;
use render_gl::ColorBuffer;
use render_gl::{DebugLines, RectMarker};
use std::collections::HashMap;
use std::collections::BTreeSet;
use failure;
use resources;

struct ControlInfo {
    _id: Ix,
    size: Option<(i32, i32)>,
    absolute_transform: Option<na::Projective3<f32>>,
    marker: Option<RectMarker>,
}

impl ControlInfo {
    pub fn new(id: Ix, _parent_id: Option<Ix>) -> ControlInfo {
        ControlInfo {
            _id: id,
            size: None,
            absolute_transform: None,
            marker: None,
        }
    }

    pub fn update_size(&mut self, size: Option<(i32, i32)>) {
        self.size = size;
    }

    pub fn update_transform(&mut self, absolute_transform: &na::Projective3<f32>) {
        self.absolute_transform = Some(absolute_transform.clone());
    }

    pub fn flush_updates(&mut self, debug_lines: &DebugLines) {
        match (self.marker.is_some(), self.size, self.absolute_transform) {
            (false, Some(wh), Some(t)) => {
                self.marker = Some(debug_lines.rect_marker(
                    t * na::Translation3::new(1.0, 0.0, 0.0),
                    na::Vector2::new((wh.0 - 1) as f32, (wh.1 - 1) as f32),
                    na::Vector4::new(1.0, 0.5, 0.2, 1.0),
                ))
            },
            (true, Some(wh), Some(t)) => {
                let marker = self.marker.as_mut().unwrap();
                marker.update_size_and_color(
                    na::Vector2::new((wh.0 - 1) as f32, (wh.1 - 1) as f32),
                    na::Vector4::new(1.0, 0.5, 0.2, 1.0),
                );
                marker.update_transform(t * na::Translation3::new(1.0, 0.0, 0.0));
            },
            (false, _, _) => {
            },
            (true, _, _) => {
                self.marker = None
            },
        }
    }
}

pub struct Interface {
    fill: Leaf<controls::Fill>,
    events: Events,
    controls: HashMap<Ix, ControlInfo>,
    event_read_buffer: Vec<Effect>,
    flush_updates_set: BTreeSet<Ix>,

    debug_lines: DebugLines,
}

impl Interface {
    pub fn new(gl: &gl::Gl, resources: &resources::Resources, size: ElementSize) -> Result<Interface, failure::Error> {
        let tree = Tree::new();

        let events = tree.events();
        let fill = tree.create_root(controls::Fill::new());

        fill.resize(size);

        Ok(Interface {
            fill,
            events,
            controls: HashMap::new(),
            event_read_buffer: Vec::new(),
            flush_updates_set: BTreeSet::new(),
            debug_lines: DebugLines::new(gl, resources)?,
        })
    }

    pub fn resize(&mut self, size: ElementSize) {
        self.fill.resize(size);
    }

    fn process_events(&mut self) {
        self.events.drain_into(&mut self.event_read_buffer);
        self.flush_updates_set.clear();

        for event in self.event_read_buffer.drain(..) {
            match event {
                Effect::Add { id, parent_id } => {
                    self.controls.insert(id, ControlInfo::new(id, parent_id));
                    self.flush_updates_set.insert(id);
                }
                Effect::Resize { id, size } => {
                    self.controls.get_mut(&id).map(|c| c.update_size(size));
                    self.flush_updates_set.insert(id);
                }
                Effect::Transform { id, absolute_transform } => {
                    self.controls.get_mut(&id).map(|c| c.update_transform(&absolute_transform));
                    self.flush_updates_set.insert(id);
                }
                Effect::Remove { id } => {
                    self.controls.remove(&id).expect("process_events: self.controls.remove(&id)");
                }
            }
        }

        for id in &self.flush_updates_set {
            let debug_lines = &self.debug_lines;
            self.controls.get_mut(id).map(|c| c.flush_updates(debug_lines));
        }
    }

    pub fn update(&mut self, _delta: f32) {
        self.process_events()
    }

    pub fn mouse_move(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_down(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_up(&mut self, _x: i32, _y: i32) {}

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        self.debug_lines.render(gl, target, vp_matrix);
    }
}