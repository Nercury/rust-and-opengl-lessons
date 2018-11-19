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
                .with_slide(TextSlide::new("First slide"))
                .with_slide(TextSlide::new("Second slide"))
                .with_slide(TextSlide::new("Yet another slide"))
                .with_slide(CombinedSlide)
        );
    }
}

pub struct CombinedSlide;

impl Element for CombinedSlide {
    fn inflate(&mut self, base: &mut Base) {
        base.add(TextSlide::new("Combined"));
        base.add(TextSlide::new("Slide"));
    }
}

pub struct Presentation {
    slides: Vec<Box<Element>>,
    num_elements: usize,
    slide_index: usize,
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation {
            slides: vec![],
            num_elements: 0,
            slide_index: 0,
        }
    }

    pub fn with_slide<E: Element + 'static>(mut self, slide: E) -> Self {
        self.slides.push(Box::new(slide) as Box<Element>);
        self.num_elements += 1;
        self
    }
}

impl Element for Presentation {
    fn inflate(&mut self, base: &mut Base) {
        for slide in self.slides.drain(..) {
            base.add_boxed(slide);
        }
        base.enable_actions(true);
    }

    fn resize(&mut self, base: &mut Base) {
        if self.num_elements == 0 {
            return base.layout_empty();
        }

        let current_slide_index = self.slide_index;
        let size = match base.box_size() {
            BoxSize::Hidden => return base.layout_empty(),
            BoxSize::Auto => {
                let mut resolved_child_size = None;

                base.children_mut(|i, mut child| {
                    if i == current_slide_index {
                        resolved_child_size = child.element_resize(BoxSize::Auto);
                    } else {
                        child.element_resize(BoxSize::Hidden);
                    }
                });

                resolved_child_size
            },
            BoxSize::Fixed { w, h } => {
                let mut resolved_child_size = None;

                base.children_mut(|i, mut child| {
                    if i == current_slide_index {
                        resolved_child_size = child.element_resize(BoxSize::Fixed { w, h });
                    } else {
                        child.element_resize(BoxSize::Hidden);
                    }
                });

                resolved_child_size
            },
        };

        base.resolve_size(size);
    }

    fn action(&mut self, base: &mut Base, action: UiAction) {
        match action {
            UiAction::NextSlide => {
                debug!("next slide");
                if self.slide_index < self.num_elements - 1 {
                    self.slide_index += 1;
                    base.invalidate_size();
                }
            },
            UiAction::PreviousSlide => {
                debug!("previous slide");
                if self.slide_index > 0 {
                    self.slide_index -= 1;
                    base.invalidate_size();
                }
            },
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
            text.set_position(size.w as f32 / 2.0 - text_size.width / 2.0, size.h as f32 / 2.0 + text_size.descent + text_size.height / 2.0);
        }

        base.resolve_size(resolved_size);
    }
}



