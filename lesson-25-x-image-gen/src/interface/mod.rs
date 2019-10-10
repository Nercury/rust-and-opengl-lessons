use failure;
use gl;
use crate::na;
use crate::render_gl::data;
use crate::render_gl::ColorBuffer;
use crate::render_gl::{DebugLines, RectMarker};
use crate::render_gl::{Flatlander, Alphabet, FlatlanderVertex, FlatlandGroup, FlatlandItem};
use resources;
use std::collections;
use metrohash::MetroHashSet;
use metrohash::MetroHashMap;
use ui::*;

pub mod controls;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum ControlId {
    Node(Ix),
    Text(usize),
}

struct ControlInfo {
    _id: ControlId,
    size: Option<(i32, i32)>,
    absolute_transform: Option<na::Projective3<f32>>,
    marker: Option<RectMarker>,
    flatland_group_data: Option<(Alphabet, Vec<FlatlandItem>)>,
    flatland_group: Option<FlatlandGroup>,
    color: Option<na::Vector4<u8>>,
}

impl ControlInfo {
    pub fn new(id: ControlId) -> ControlInfo {
        ControlInfo {
            _id: id,
            size: None,
            absolute_transform: None,
            marker: None,
            flatland_group_data: None,
            flatland_group: None,
            color: None,
        }
    }

    pub fn new_flatlander(id: ControlId, alphabet: Alphabet, items: Vec<FlatlandItem>) -> ControlInfo {
        ControlInfo {
            _id: id,
            size: None,
            absolute_transform: None,
            marker: None,
            flatland_group_data: Some((alphabet, items)),
            flatland_group: None,
            color: None,
        }
    }

    pub fn update_size(&mut self, size: Option<(i32, i32)>) {
        self.size = size;
    }

    pub fn update_transform(&mut self, absolute_transform: Option<na::Projective3<f32>>) {
        self.absolute_transform = absolute_transform;
    }

    pub fn update_color(&mut self, color: Option<na::Vector4<u8>>) {
        self.color = color;
    }

    pub fn flush_updates(&mut self, debug_lines: &DebugLines) {
        match (self.marker.is_some(), self.size, self.absolute_transform) {
            (false, Some(wh), Some(t)) => {
                self.marker = Some(debug_lines.rect_marker(
                    t * na::Translation3::new(1.0, 0.0, 0.0),
                    na::Vector2::new((wh.0 - 1) as f32, (wh.1 - 1) as f32),
                    na::Vector4::new(0.0, 0.5, 1.0, 1.0),
                ))
            }
            (true, Some(wh), Some(t)) => {
                let marker = self.marker.as_mut().unwrap();
                marker.update_size_and_color(
                    na::Vector2::new((wh.0 - 1) as f32, (wh.1 - 1) as f32),
                    na::Vector4::new(0.0, 0.5, 1.0, 1.0),
                );
                marker.update_transform(t * na::Translation3::new(1.0, 0.0, 0.0));
            }
            (false, _, _) => {}
            (true, _, _) => self.marker = None,
        }

        match (self.flatland_group.is_some(), &self.flatland_group_data, self.absolute_transform, self.color) {
            (false, &Some((ref alphabet, ref items)), Some(t), Some(color)) => {
                self.flatland_group = Some(
                    FlatlandGroup::new(&t, color, alphabet.clone(), items.clone())
                );
            }
            (true, Some((ref _alphabet, ref items)), Some(t), Some(color)) => {
                let g = self.flatland_group.as_mut().unwrap();
                g.update_items(items.iter());
                g.update_transform(&t);
                g.update_color(color);
            },
            (false, _, _, _) => {},
            (true, _, _, _) => {
                self.flatland_group = None
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum AlphabetKey {
    Font(usize),
}

pub struct Interface {
    tree: Tree,
    fonts: Fonts,
    fill: Leaf<controls::rust_fest::RustFest>,
    events: Events,
    controls: MetroHashMap<ControlId, ControlInfo>,
    event_read_buffer: Vec<Effect>,
    flush_updates_set: MetroHashSet<ControlId>,

    debug_lines: DebugLines,
    flatlander: Flatlander,

    alphabets: MetroHashMap<AlphabetKey, Alphabet>,
}

impl Interface {
    pub fn new(
        gl: &gl::Gl,
        resources: &resources::Resources,
        size: BoxSize,
        window_scale: f32
    ) -> Result<Interface, failure::Error> {
        let tree = Tree::new(resources);
        let fonts = tree.fonts();

        let events = tree.events();
        let fill = tree.create_root(controls::rust_fest::RustFest::new(), window_scale);

        fill.resize(size, window_scale);

        Ok(Interface {
            tree,
            fonts,
            fill,
            events,
            controls: MetroHashMap::default(),
            event_read_buffer: Vec::new(),
            flush_updates_set: MetroHashSet::default(),
            debug_lines: DebugLines::new(gl, resources)?,
            flatlander: Flatlander::new(gl, resources)?,
            alphabets: MetroHashMap::default(),
        })
    }

    pub fn resize(&mut self, size: BoxSize, window_scale: f32) {
        self.fill.resize(size, window_scale);
    }

    fn process_events(&mut self) {
        self.events.drain_into(&mut self.event_read_buffer);
        self.flush_updates_set.clear();

        let mut glyph_buffer = Vec::new();

        for event in self.event_read_buffer.drain(..) {
            match event {
                Effect::Add { id, .. } => {
                    self.controls.insert(ControlId::Node(id), ControlInfo::new(ControlId::Node(id)));
                    self.flush_updates_set.insert(ControlId::Node(id));
                }
                Effect::Resize { id, size } => {
                    self.controls.get_mut(&ControlId::Node(id)).map(|c| c.update_size(size));
                    self.flush_updates_set.insert(ControlId::Node(id));
                }
                Effect::Transform {
                    id,
                    absolute_transform,
                } => {
                    self.controls
                        .get_mut(&ControlId::Node(id))
                        .map(|c| c.update_transform(absolute_transform));
                    self.flush_updates_set.insert(ControlId::Node(id));
                }
                Effect::Remove { id } => {
                    self.controls
                        .remove(&ControlId::Node(id))
                        .expect("process_events: self.controls.remove(&id)");
                }
                Effect::TextAdd { buffer } => {

                    let alphabet = match self.alphabets.entry(AlphabetKey::Font(buffer._font_id)) {
                        collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                        collections::hash_map::Entry::Vacant(mut e) => e.insert(self.flatlander.create_alphabet()),
                    };

                    let buffer = self.fonts.buffer_from_id(buffer._id).expect("buffer missing: self.fonts.buffer_from_id(buffer.id)");

                    use lyon_path::default::Path;

                    let mut builder = Path::builder();
                    glyph_buffer.clear();
                    buffer.glyphs(&mut glyph_buffer);

                    let mut flatland_group_items = Vec::new();

                    let mut x = 0;
                    let mut y = 0;

                    for glyph in glyph_buffer.iter() {
                        let ix = ensure_glyph_is_in_alphabet_and_return_index(&mut builder, alphabet, buffer.font(), glyph.id);
                        flatland_group_items.push(FlatlandItem {
                            alphabet_entry_index: ix,
                            x_offset: x,
                            y_offset: y,
                        });

                        x += glyph.x_advance + glyph.x_offset;
                        y += glyph.y_advance + glyph.y_offset;
                    }

                    self.controls.insert(ControlId::Text(buffer.id()), ControlInfo::new_flatlander(
                        ControlId::Text(buffer.id()), alphabet.clone(), flatland_group_items
                    ));
                    self.flush_updates_set.insert(ControlId::Text(buffer.id()));
                }
                Effect::TextUpdate { buffer_id, absolute_transform, color } => {
                    self.controls
                        .get_mut(&ControlId::Text(buffer_id))
                        .map(|c| {
                            c.update_transform(absolute_transform);
                            c.update_color(Some(color));
                        });
                    self.flush_updates_set.insert(ControlId::Text(buffer_id));
                }
                Effect::TextRemove { buffer_id } => {
                    if let None = self.controls.remove(&ControlId::Text(buffer_id)) {
                        warn!("tried to remove nonexisting flatland group {}", buffer_id);
                    }
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

    pub fn send_action(&mut self, action: UiAction) {
        self.tree.send_action(action);
    }

    pub fn mouse_move(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_down(&mut self, _x: i32, _y: i32) {}

    pub fn mouse_up(&mut self, _x: i32, _y: i32) {}

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        let flipped_matrix = vp_matrix * na::Matrix4::new_nonuniform_scaling(&[1.0, -1.0, 1.0].into());

        self.debug_lines.render(gl, target, &flipped_matrix);
        self.flatlander.render(gl, target, &flipped_matrix);
    }

    pub fn toggle_wireframe(&mut self) {
        self.flatlander.toggle_wireframe()
    }

    pub fn toggle_bounds(&mut self) {
        self.debug_lines.toggle()
    }
}

fn ensure_glyph_is_in_alphabet_and_return_index(builder: &mut lyon_path::default::Builder, alphabet: &mut Alphabet, font: &Font, glyph_id: u32) -> usize {
    if let Some(index) = alphabet.get_entry_index(glyph_id) {
        return index;
    }

    trace!("tessellate glyph {} from {:?} font", glyph_id, font.full_name());

    use lyon_path::builder::{FlatPathBuilder};
    use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

    font.outline(glyph_id, ui::HintingOptions::None, builder).expect("outline failed");
    let path = builder.build_and_reset();

    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<FlatlanderVertex, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    {
        // Compute the tessellation.
        tessellator.tessellate_path(
            path.path_iter(),
            &FillOptions::default().with_tolerance(10.0),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                FlatlanderVertex {
                    pos: data::f16_f16::from((vertex.position.x, vertex.position.y)),
                    normal: data::f16_f16::from((vertex.normal.x, vertex.normal.y)),
                }
            }),
        ).unwrap();
    }

    alphabet.add_entry(glyph_id, geometry.vertices, geometry.indices)
}