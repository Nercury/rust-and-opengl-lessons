use fonts::*;
use *;
use std::cell::RefCell;
use std::rc::Rc;
pub use self::shared::ModificationLogEntry;

#[derive(Copy, Clone, Debug)]
pub struct GlyphMeasurement {
    pub id: u32,
    pub cluster: u32,
    pub byte_offset: u32,
    pub len: u32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(Clone)]
pub struct TextMeasurement {
    size: f32,
    font_scale: f32,

    metrics: Option<Measurement>,
    glyph_positions: Vec<GlyphPosition>,

    shared: Rc<RefCell<shared::InnerPrimitives>>,
    buffer: Buffer,
}

impl TextMeasurement {
    fn update_metrics(&mut self) {
        self.glyph_positions.clear();
        self.metrics = self.buffer.measure(&mut self.glyph_positions);
    }

    pub fn measure(&mut self) -> Option<Measurement> {
        if self.metrics.is_none() {
            self.update_metrics();
        }

        if let Some(m) = self.metrics {
            let shared =  self.shared.borrow();
            let ws = shared.get_window_scale();
            let s = self.scale() * ws;
            return Some(Measurement {
                ascent: m.ascent * s,
                descent: m.descent * s,
                width: m.width * s,
                cap_height: m.cap_height * s,
                x_height: m.x_height * s,
                line_gap: m.line_gap * s,
                height: m.height * s,
            });
        }

        None
    }

    pub fn glyph_positions<'r>(&'r mut self) -> impl Iterator<Item = GlyphMeasurement> + 'r {
        if self.metrics.is_none() {
            self.update_metrics();
        }

        let shared =  self.shared.borrow();
        let ws = shared.get_window_scale();
        let s = self.scale() * ws;

        self.glyph_positions.iter()
            .map(move |p| GlyphMeasurement {
                id: p.id,
                cluster: p.cluster,
                byte_offset: p.byte_offset,
                len: p.len,
                x_advance: p.x_advance as f32 * s,
                y_advance: p.y_advance as f32 * s,
                x_offset: p.x_offset as f32 * s,
                y_offset: p.y_offset as f32 * s,
            })
    }

    pub fn scale(&self) -> f32 {
        self.font_scale * self.size
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
}

pub struct Text {
    measurement: TextMeasurement,

    position: na::Vector3<f32>,
    origin: na::Vector3<f32>,
    transform: na::Projective3<f32>,
    color: na::Vector4<u8>,
    slot: shared::PrimitiveSlot,
    hidden: bool,
}

impl Text {
    pub fn set_transform(&mut self, transform: &na::Projective3<f32>) {
        self.transform = transform.clone();
        self.update_transform();
    }

    pub fn set_size(&mut self, size: f32) {
        self.measurement.set_size(size);
        self.update_transform();
    }

    pub fn set_hidden(&mut self, value: bool) {
        self.hidden = value;
        self.update_transform();
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = [x, y, 0.0].into();
        self.update_transform();
    }

    pub fn set_origin(&mut self, x: f32, y: f32) {
        self.origin = [x, y, 0.0].into();
        self.update_transform();
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color = [r, g, b, a].into();
        self.update_transform();
    }

    fn update_transform(&self) {
        if self.hidden {
            let mut shared =  self.measurement.shared.borrow_mut();
            shared.set_text_transform(self.slot, None);
        } else {
            let scale = self.measurement.scale();

            let mut shared = self.measurement.shared.borrow_mut();
            shared.set_text_transform(self.slot, Some(
                self.transform
                    * na::convert::<_, na::Projective3<_>>(na::Translation3::new(self.position.x, self.position.y, self.position.z))
                    * na::convert::<_, na::Projective3<_>>(na::Translation3::new(-self.origin.x, -self.origin.y, -self.origin.z))
                    * na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), scale))
            ));
        }
    }

    pub fn measurement(&mut self) -> &mut TextMeasurement {
        &mut self.measurement
    }

    pub fn into_measurement(self) -> TextMeasurement {
        self.measurement.clone()
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        self.measurement.shared.borrow_mut().delete_text_buffer(self.slot);
    }
}

#[derive(Clone)]
pub struct Primitives {
    fonts: Fonts,
    pub (crate) shared: Rc<RefCell<shared::InnerPrimitives>>,
}

impl Primitives {
    pub (crate) fn new(fonts: &Fonts, window_scale: f32) -> Primitives {
        Primitives {
            fonts: fonts.clone(),
            shared: Rc::new(RefCell::new(shared::InnerPrimitives::new(window_scale))),
        }
    }

    pub fn text<P: ToString>(&mut self, text: P, bold: bool, italic: bool, monospaced: bool, color: na::Vector4<u8>) -> Option<Text> {

        let text = text.to_string();
        let mut properties = Properties::new();
        if bold {
            properties.weight(Weight::BOLD);
        }
        if italic {
            properties.style(Style::Italic);
        }

        let font = self.fonts.find_best_match( if monospaced {
            &[FamilyName::Monospace]
        } else {
            &[FamilyName::SansSerif]
        }, &properties);

        if let Some(font) = font {
            let size = 48.0;
            let metrics = font.metrics();

            let font_scale = 1.0 / metrics.units_per_em as f32;
            let scale = font_scale * size;

            let text_len = text.len();

            let (slot, buffer) = {
                let mut shared = self.shared.borrow_mut();
                shared.create_text_buffer(
                    &font, text.clone(),
                    Some(na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), scale))),
                    color
                )
            };

            return Some(Text {
                measurement: TextMeasurement {
                    metrics: None,
                    glyph_positions: Vec::with_capacity(text_len),

                    size,
                    font_scale,
                    shared: self.shared.clone(),
                    buffer,
                },

                transform: na::Projective3::<f32>::identity(),
                position: na::zero(),
                origin: na::zero(),
                slot,

                hidden: false,

                color,
            });
        }

        None
    }
}

mod shared {
    use na;
    use fonts::*;
    use slotmap;

    new_key_type! { pub struct PrimitiveSlot; }

    #[derive(Copy, Clone)]
    pub struct PrimitiveSlotKeyData {
    }

    pub struct PrimitiveSlotData {
        kind: PrimitiveKind,
        invalidated: bool,
        font_transform: Option<na::Projective3<f32>>,
    }

    impl PrimitiveSlotData {
        pub fn update_buffer(&mut self, window_scale: f32) {
            match self.kind {
                PrimitiveKind::TextBuffer(ref mut b) => b.set_transform(
                    self.font_transform.map(|transform| PrimitiveSlotData::calc_transform(&transform, window_scale))
                ),
            }
        }

        #[inline(always)]
        pub fn calc_transform(transform: &na::Projective3<f32>, window_scale: f32) -> na::Projective3<f32> {
                transform
                    * na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), window_scale))
        }
    }

    pub enum PrimitiveKind {
        TextBuffer(Buffer),
    }

    pub enum ModificationLogEntry {
        Added { buffer: Buffer },
        Removed { buffer_id: usize },
    }

    pub struct InnerPrimitives {
        pub primitive_slots: slotmap::HopSlotMap<PrimitiveSlot, PrimitiveSlotKeyData>,
        pub primitive_data: slotmap::SparseSecondaryMap<PrimitiveSlot, PrimitiveSlotData>,

        pub modification_log: Vec<ModificationLogEntry>,

        pub invalidated: bool,
        pub window_scale: f32,
    }

    impl InnerPrimitives {
        pub fn new(window_scale: f32) -> InnerPrimitives {
            InnerPrimitives {
                primitive_slots: slotmap::HopSlotMap::with_key(),
                primitive_data: slotmap::SparseSecondaryMap::new(),

                modification_log: Vec::with_capacity(32),

                invalidated: false,
                window_scale,
            }
        }

        pub fn get_window_scale(&self) -> f32 {
            self.window_scale
        }

        pub fn set_window_scale(&mut self, window_scale: f32) {
            self.window_scale = window_scale;
            self.invalidated = true;

            for (_, v) in self.primitive_data.iter_mut() {
                v.invalidated = true;
                v.update_buffer(window_scale);
            }
        }

        pub fn set_text_transform(&mut self, slot: PrimitiveSlot, transform: Option<na::Projective3<f32>>) {
            if let Some(data) = self.primitive_data.get_mut(slot) {
                self.invalidated = true;
                data.invalidated = true;
                data.font_transform = transform;
                data.update_buffer(self.window_scale);
            }
        }

        pub fn create_text_buffer<P: ToString>(&mut self, font: &Font, text: P, font_transform: Option<na::Projective3<f32>>, color: na::Vector4<u8>) -> (PrimitiveSlot, Buffer) {
            let buffer = font.create_buffer(text, font_transform.map(|t| PrimitiveSlotData::calc_transform(&t, self.window_scale)), color);

            let data = PrimitiveSlotData {
                invalidated: true,
                kind: PrimitiveKind::TextBuffer(buffer.clone()),
                font_transform,
            };

            let slot = self.primitive_slots.insert(PrimitiveSlotKeyData {});
            self.modification_log.push(ModificationLogEntry::Added { buffer: buffer.clone() });

            self.primitive_data.insert(slot, data);
            self.invalidated = true;

            (slot, buffer)
        }

        pub fn delete_text_buffer(&mut self, slot: PrimitiveSlot) {
            if let Some(data) = self.primitive_data.remove(slot) {
                self.invalidated = true;
                match data.kind {
                    PrimitiveKind::TextBuffer(b) => {
                        self.modification_log.push( ModificationLogEntry::Removed { buffer_id: b.id() });
                    },
                };
            }

            self.primitive_slots.remove(slot);
        }

        pub fn invalidate_text_buffer(&mut self, slot: PrimitiveSlot) {
            if let Some(data) = self.primitive_data.get_mut(slot) {
                self.invalidated = true;
                data.invalidated = true;
            }
        }

        pub fn modified_buffers<'r>(&'r mut self) -> impl Iterator<Item = ModificationLogEntry> + 'r {
            self.modification_log.drain(..)
        }

        pub (crate) fn buffers_keep_invalidated<'r>(&'r mut self) -> impl Iterator<Item = &'r Buffer> + 'r {
            self.primitive_data
                .iter_mut()
                .filter_map(|(_, v)| {
                    match v.kind {
                        PrimitiveKind::TextBuffer(ref b) => Some(b)
                    }
                })
        }

        pub (crate) fn buffers<'r>(&'r mut self) -> impl Iterator<Item = &'r Buffer> + 'r {
            self.invalidated = false;

            self.primitive_data
                .iter_mut()
                .filter_map(|(_, v)| {
                    v.invalidated = false;
                    match v.kind {
                        PrimitiveKind::TextBuffer(ref b) => Some(b)
                    }
                })
        }

        pub (crate) fn only_invalidated_buffers<'r>(&'r mut self) -> impl Iterator<Item = &'r Buffer> + 'r {
            self.invalidated = false;

            self.primitive_data
                .iter_mut()
                .filter_map(|(_, v)| {
                    if v.invalidated {
                        v.invalidated = false;
                        match v.kind {
                            PrimitiveKind::TextBuffer(ref b) => Some(b)
                        }
                    } else {
                        None
                    }
                })
        }
    }
}
