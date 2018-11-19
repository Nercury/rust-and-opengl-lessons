use ui::*;
use na;

pub struct RustFest {

}

impl RustFest {
    pub fn new() -> RustFest {
        RustFest {}
    }
}

impl Element for RustFest {
    fn inflate(&mut self, base: &mut Base) {
        base.add(
            Presentation::new()
                .with_slide(TextSlide::new("The lazy fox jumps over a quick dog"))
        );
    }
}


pub struct Presentation {
    slides: Vec<Box<Element>>,
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation {
            slides: vec![],
        }
    }

    pub fn with_slide<E: Element + 'static>(mut self, slide: E) -> Self {
        self.slides.push(Box::new(slide) as Box<Element>);
        self
    }
}

impl Element for Presentation {
    fn inflate(&mut self, base: &mut Base) {
        for slide in self.slides.drain(..) {
            base.add_boxed(slide);
        }
    }
}

pub struct TextSlide {
    text: Option<primitives::Text>,
    text_string: String,
}

impl TextSlide {
    pub fn new(text: &str) -> TextSlide {
        TextSlide {
            text: None,
            text_string: text.into(),
        }
    }
}

impl Element for TextSlide {
    fn inflate(&mut self, base: &mut Base) {
        let mut text = base.primitives().text(self.text_string.clone()).expect("failed to create text");
        text.set_size(60.0);
        self.text = Some(text);
    }

    fn resize(&mut self, base: &mut Base) {
        let text_size = self.text.as_ref().unwrap().measure();

        let box_size = base.box_size();
        let resolved_size = match box_size {
            BoxSize::Hidden => None,
            BoxSize::Auto => Some(ResolvedSize {
                w: text_size.map(|m| m.width).unwrap_or(0.0).round() as i32,
                h: text_size.map(|m| m.height).unwrap_or(0.0).round() as i32
            }),
            BoxSize::Fixed { w, h, .. } => Some(ResolvedSize { w, h }),
        };

        if let (Some(size), Some(text_size)) = (resolved_size, text_size) {
            let text = self.text.as_mut().unwrap();
            text.set_position(size.w as f32 / 2.0 - text_size.width / 2.0, size.h as f32 / 2.0 + text_size.x_height / 2.0);
        }

        base.resolve_size(resolved_size);
    }
}



