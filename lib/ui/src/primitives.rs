use queues::*;
use fonts::*;
use *;

pub struct PrimitivesMutator<'a> {
    next_primitive_id: &'a mut Ix,
    fonts: &'a mut Fonts,
    primitives: &'a mut Primitives,
    queues: &'a mut Queues,
}

impl<'a> PrimitivesMutator<'a> {
    pub fn new<'r>(next_primitive_id: &'r mut Ix, fonts: &'r mut Fonts, primitives: &'r mut Primitives, queues: &'r mut Queues) -> PrimitivesMutator<'r> {
        PrimitivesMutator {
            next_primitive_id, fonts, primitives, queues
        }
    }

    pub fn text<P: ToString>(&mut self, text: P) -> Option<Text> {
        let font = self.fonts.find_best_match(&[FamilyName::SansSerif],
                                   &Properties::new());

        if let Some(font) = font {
            let buffer = font.create_buffer(text);
            let weak_ref = buffer.weak_ref();
            self.primitives.buffers.push(buffer.clone());

            self.queues.send(Effect::TextAdd { buffer: weak_ref });

            return Some(Text {
                id: self.next_primitive_id.inc(),
                buffer,
            });
        }

        None
    }
}

pub struct Text {
    id: Ix,
    buffer: Buffer,
}

pub struct Primitives {
    buffers: Vec<Buffer>,
}

impl Primitives {
    pub fn new() -> Primitives {
        Primitives {
            buffers: Vec::new(),
        }
    }
}

