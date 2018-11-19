use fonts::*;
use *;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Text {
    height: f32,
    font_scale: f32,
    slot: shared::PrimitiveSlot,
    shared: Rc<RefCell<shared::InnerPrimitives>>,
}

impl Text {
    pub fn set_transform(&mut self, transform: &na::Projective3<f32>) {
        let mut shared =  self.shared.borrow_mut();
        shared.set_text_transform(self.slot, transform);
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        self.shared.borrow_mut().delete_text_buffer(self.slot);
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

    pub fn text<P: ToString>(&mut self, text: P) -> Option<Text> {

        let font = self.fonts.find_best_match(&[FamilyName::SansSerif],
                                   &{ let mut p = Properties::new(); p.weight(Weight::BOLD); p });

        if let Some(font) = font {
            let height = 48.0;
            let metrics = font.metrics();

            let font_scale = 1.0 / metrics.units_per_em as f32;

            let window_scale = self.shared.borrow_mut().get_window_scale();
            let scale = font_scale * height * window_scale;

            let slot = {
                let buffer = font.create_buffer(text, &na::Projective3::<f32>::identity());
                let mut shared = self.shared.borrow_mut();

                shared.create_text_buffer(buffer)
            };

            return Some(Text {
                height,
                font_scale,
                slot,
                shared: self.shared.clone(),
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
        transform: na::Projective3<f32>,
    }

    impl PrimitiveSlotData {
        pub fn update_buffer(&mut self, window_scale: f32) {
            match self.kind {
                PrimitiveKind::TextBuffer(ref mut b) => b.set_transform(
                    &(
                        self.transform
                        * na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), window_scale))
                    )
                ),
            }
        }
    }

    pub enum PrimitiveKind {
        TextBuffer(Buffer),
    }

    pub struct InnerPrimitives {
        pub primitive_slots: slotmap::HopSlotMap<PrimitiveSlot, PrimitiveSlotKeyData>,
        pub primitive_data: slotmap::SparseSecondaryMap<PrimitiveSlot, PrimitiveSlotData>,

        pub added_buffers: Vec<Buffer>,
        pub removed_buffer_ids: Vec<usize>,

        pub invalidated: bool,
        pub window_scale: f32,
    }

    impl InnerPrimitives {
        pub fn new(window_scale: f32) -> InnerPrimitives {
            InnerPrimitives {
                primitive_slots: slotmap::HopSlotMap::with_key(),
                primitive_data: slotmap::SparseSecondaryMap::new(),

                added_buffers: Vec::with_capacity(32),
                removed_buffer_ids: Vec::with_capacity(32),

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

        pub fn set_text_transform(&mut self, slot: PrimitiveSlot, transform: &na::Projective3<f32>) {
            if let Some(data) = self.primitive_data.get_mut(slot) {
                self.invalidated = true;
                data.invalidated = true;
                data.transform = transform.clone();
                data.update_buffer(self.window_scale);
            }
        }

        pub fn create_text_buffer(&mut self, buffer: Buffer) -> PrimitiveSlot {
            let slot = self.primitive_slots.insert(PrimitiveSlotKeyData {});
            self.added_buffers.push(buffer.clone());
            self.primitive_data.insert(slot, PrimitiveSlotData {
                invalidated: true,
                kind: PrimitiveKind::TextBuffer(buffer),
                transform: na::Projective3::<f32>::identity(),
            });
            self.invalidated = true;

            slot
        }

        pub fn delete_text_buffer(&mut self, slot: PrimitiveSlot) {
            if let Some(data) = self.primitive_data.remove(slot) {
                self.invalidated = true;
                match data.kind {
                    PrimitiveKind::TextBuffer(b) => self.removed_buffer_ids.push(b.id()),
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

        pub fn added_text_buffers<'r>(&'r mut self) -> impl Iterator<Item = Buffer> + 'r {
            self.added_buffers.drain(..)
        }

        pub fn removed_text_buffers<'r>(&'r mut self) -> impl Iterator<Item = usize> + 'r {
            self.removed_buffer_ids.drain(..)
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
