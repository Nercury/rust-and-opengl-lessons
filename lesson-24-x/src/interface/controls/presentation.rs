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
        text.set_transform(
            &(na::convert::<_, na::Projective3<_>>(na::Translation3::new(0.0, 80.0, 0.0)) // translation
                * na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), 0.05 * 1.0))) // scale
        );
        self.text = Some(text);
    }

    fn resize(&mut self, base: &mut Base) {
        let box_size = base.box_size();
        base.resolve_size(match box_size {
            BoxSize::Hidden => None,
            BoxSize::Auto => Some(ResolvedSize { w: 500, h: 30 }),
            BoxSize::Fixed { w, h, .. } => Some(ResolvedSize { w, h }),
        });
    }
}



