use fonts::*;
use *;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Text {
    slot: shared::PrimitiveSlot,
    buffer: Buffer,
    shared: Rc<RefCell<shared::InnerPrimitives>>,
}

impl Text {
    pub fn set_transform(&mut self, transform: &na::Projective3<f32>) {
        self.buffer.set_transform(transform);
        self.shared.borrow_mut().invalidate_text_buffer(self.slot);
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
    pub (crate) fn new(fonts: &Fonts) -> Primitives {
        Primitives {
            fonts: fonts.clone(),
            shared: Rc::new(RefCell::new(shared::InnerPrimitives::new())),
        }
    }

    pub fn text<P: ToString>(&mut self, text: P) -> Option<Text> {
        let font = self.fonts.find_best_match(&[FamilyName::Title("Snell Roundhand".into()), FamilyName::Serif],
                                   &{ let mut p = Properties::new(); p.weight(Weight::BOLD); p });

        if let Some(font) = font {
            let (slot, buffer) = {
                let buffer = font.create_buffer(text);
                let mut shared = self.shared.borrow_mut();

                (shared.create_text_buffer(buffer.clone()), buffer)
            };

            return Some(Text {
                slot,
                buffer,
                shared: self.shared.clone(),
            });
        }

        None
    }
}

mod shared {
    use fonts::*;
    use slotmap;

    new_key_type! { pub struct PrimitiveSlot; }

    #[derive(Copy, Clone)]
    pub struct PrimitiveSlotKeyData {
    }

    pub struct PrimitiveSlotData {
        kind: PrimitiveKind,
        invalidated: bool,
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
    }

    impl InnerPrimitives {
        pub fn new() -> InnerPrimitives {
            InnerPrimitives {
                primitive_slots: slotmap::HopSlotMap::with_key(),
                primitive_data: slotmap::SparseSecondaryMap::new(),

                added_buffers: Vec::with_capacity(32),
                removed_buffer_ids: Vec::with_capacity(32),

                invalidated: false,
            }
        }

        pub fn create_text_buffer(&mut self, buffer: Buffer) -> PrimitiveSlot {
            let slot = self.primitive_slots.insert(PrimitiveSlotKeyData {});
            self.added_buffers.push(buffer.clone());
            self.primitive_data.insert(slot, PrimitiveSlotData {
                invalidated: true,
                kind: PrimitiveKind::TextBuffer(buffer),
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
