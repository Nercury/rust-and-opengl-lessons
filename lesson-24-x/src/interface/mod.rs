use failure;
use gl;
use na;
use render_gl::data;
use render_gl::ColorBuffer;
use render_gl::{DebugLines, RectMarker};
use render_gl::{Flatlander, Alphabet, FlatlanderVertex};
use resources;
use std::collections::BTreeSet;
use std::collections::{HashMap, self};
use ui::*;
use lyon_path::default::Path;

mod controls;

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
            }
            (true, Some(wh), Some(t)) => {
                let marker = self.marker.as_mut().unwrap();
                marker.update_size_and_color(
                    na::Vector2::new((wh.0 - 1) as f32, (wh.1 - 1) as f32),
                    na::Vector4::new(1.0, 0.5, 0.2, 1.0),
                );
                marker.update_transform(t * na::Translation3::new(1.0, 0.0, 0.0));
            }
            (false, _, _) => {}
            (true, _, _) => self.marker = None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum AlphabetFeature {
    Font = 0,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct AlphabetKey {
    feature: AlphabetFeature,
    id: usize,
}

pub struct Interface {
    tree: Tree,
    fonts: Fonts,
    fill: Leaf<controls::Fill>,
    events: Events,
    controls: HashMap<Ix, ControlInfo>,
    event_read_buffer: Vec<Effect>,
    flush_updates_set: BTreeSet<Ix>,

    debug_lines: DebugLines,
    flatlander: Flatlander,

    alphabets: HashMap<AlphabetKey, Alphabet>,
}

impl Interface {
    pub fn new(
        gl: &gl::Gl,
        resources: &resources::Resources,
        size: BoxSize,
    ) -> Result<Interface, failure::Error> {
        let tree = Tree::new();
        let fonts = tree.fonts();

        let events = tree.events();
        let fill = tree.create_root(controls::Fill::new());

        fill.resize(size);

        Ok(Interface {
            tree,
            fonts,
            fill,
            events,
            controls: HashMap::new(),
            event_read_buffer: Vec::new(),
            flush_updates_set: BTreeSet::new(),
            debug_lines: DebugLines::new(gl, resources)?,
            flatlander: Flatlander::new(gl, resources)?,
            alphabets: HashMap::new(),
        })
    }

    pub fn resize(&mut self, size: BoxSize) {
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
                Effect::Transform {
                    id,
                    absolute_transform,
                } => {
                    self.controls
                        .get_mut(&id)
                        .map(|c| c.update_transform(&absolute_transform));
                    self.flush_updates_set.insert(id);
                }
                Effect::Remove { id } => {
                    self.controls
                        .remove(&id)
                        .expect("process_events: self.controls.remove(&id)");
                }
                Effect::TextAdd { buffer } => {
                    match self.alphabets.entry(AlphabetKey { feature: AlphabetFeature::Font, id: buffer.font_id }) {
                        collections::hash_map::Entry::Occupied(_) => {},
                        collections::hash_map::Entry::Vacant(mut e) => {
                            if let Some(font) = self.fonts.font_from_id(buffer.font_id) {
                                let alphabet = self.flatlander.create_alphabet();

                                println!("name: {:?}", font.full_name());

                                use lyon_path::default::Path;
                                use lyon_path::builder::{FlatPathBuilder};
                                use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

                                let mut builder = Path::builder();

                                for glyph_ix in 0..font.glyph_count() {

                                    font.outline(glyph_ix, ui::HintingOptions::None, &mut builder).expect("outline failed");
                                    let path = builder.build_and_reset();

                                    // Will contain the result of the tessellation.
                                    let mut geometry: VertexBuffers<FlatlanderVertex, u16> = VertexBuffers::new();
                                    let mut tessellator = FillTessellator::new();

                                    {
                                        // Compute the tessellation.
                                        tessellator.tessellate_path(
                                            path.path_iter(),
                                            &FillOptions::default().with_tolerance(100.0),
                                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                                                FlatlanderVertex {
                                                    pos: data::f32_f32::new(vertex.position.x, vertex.position.y),
                                                    normal: data::f32_f32::new(vertex.normal.x, vertex.normal.y),
                                                }
                                            }),
                                        ).unwrap();
                                    }

                                    alphabet.add_entry(glyph_ix, geometry.vertices, geometry.indices);
                                }

                                e.insert(alphabet);
                            }
                        },
                    }
                }
                Effect::TextRemove { buffer } => {

                }
            }
        }

        for id in &self.flush_updates_set {
            let debug_lines = &self.debug_lines;
            self.controls
                .get_mut(id)
                .map(|c| c.flush_updates(debug_lines));
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.tree.update(delta);
        self.process_events()
    }

    pub fn mouse_move(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_down(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_up(&mut self, _x: i32, _y: i32) {}

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        self.debug_lines.render(gl, target, vp_matrix);
        self.flatlander.render(gl, target, vp_matrix);
    }
}
