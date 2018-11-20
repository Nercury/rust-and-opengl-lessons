use ui::*;
use na;
use glm;

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

struct InterpolationTarget {
    rotation: na::UnitQuaternion<f32>,
    position: na::Vector3<f32>,
    t: f32,
}

struct ElementInfo {
    rotation: na::UnitQuaternion<f32>,
    position: na::Vector3<f32>,
    target: Option<InterpolationTarget>,
}

impl ElementInfo {
    pub fn new() -> ElementInfo {
        ElementInfo {
            rotation: na::UnitQuaternion::<f32>::identity(),
            position: [0.0, 0.0, 0.0].into(),
            target: None,
        }
    }

    pub fn transform(&self) -> na::Projective3<f32> {
        if let Some(ref target) = self.target {
            let alpha = smootherstep(0.0, 1.0, target.t);
            let interpolated_position = self.position.lerp(&target.position, alpha);
            let interpolated_rotation = self.rotation.nlerp(&target.rotation, alpha);

            na::convert::<_, na::Projective3<_>>( na::Translation3::new(interpolated_position.x, interpolated_position.y, interpolated_position.z))
                * na::convert::<_, na::Projective3<_>>(interpolated_rotation)
        } else {
            na::convert::<_, na::Projective3<_>>( na::Translation3::new(self.position.x, self.position.y, self.position.z))
                * na::convert::<_, na::Projective3<_>>(self.rotation)
        }
    }

    pub fn transition_from(&mut self, source_pos: &na::Vector3<f32>) {
        self.target = Some(InterpolationTarget {
            rotation: na::UnitQuaternion::<f32>::identity(),
            position: [0.0, 0.0, 0.0].into(),
            t: 0.0,
        });
        self.position = *source_pos;
    }
}

fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = na::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

pub struct Presentation {
    slides: Vec<Box<Element>>,
    num_elements: usize,
    slide_index: usize,
    element_info: Vec<ElementInfo>,
    last_size: ResolvedSize,
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation {
            slides: vec![],
            num_elements: 0,
            slide_index: 0,
            element_info: vec![],
            last_size: ResolvedSize::zero(),
        }
    }

    pub fn with_slide<E: Element + 'static>(mut self, slide: E) -> Self {
        self.slides.push(Box::new(slide) as Box<Element>);
        self.num_elements += 1;
        self.element_info.push(ElementInfo::new());
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

                    child.element_transform(&self.element_info[i].transform());
                });

                resolved_child_size
            },
        };

        if let Some(size) = size {
            self.last_size = size;
        }
        base.resolve_size(size);
    }

    fn update(&mut self, base: &mut Base, delta: f32) {
        let mut someone_updating = false;

        base.children_mut(|i, mut child| {
            let (end, update) = match self.element_info[i].target {
                Some(ref mut target) => {
                    target.t += 0.5 * delta;
                    (target.t >= 1.0, true)
                },
                None => (false, false),
            };

            if update {
                if end {
                    self.element_info[i].position = self.element_info[i].target.as_mut().unwrap().position;
                    self.element_info[i].rotation = self.element_info[i].target.as_mut().unwrap().rotation;
                    self.element_info[i].target = None;
                }

                child.element_transform(&self.element_info[i].transform());

                someone_updating = true;
            }
        });

        if !someone_updating {
            base.enable_update(false);
        }
    }

    fn action(&mut self, base: &mut Base, action: UiAction) {
        match action {
            UiAction::NextSlide => {
                debug!("next slide");
                if self.slide_index < self.num_elements - 1 {
                    self.slide_index += 1;

                    self.element_info[self.slide_index].transition_from(
                        &[self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    base.enable_update(true);
                    base.invalidate_size();
                }
            },
            UiAction::PreviousSlide => {
                debug!("previous slide");
                if self.slide_index > 0 {
                    self.slide_index -= 1;

                    self.element_info[self.slide_index].transition_from(
                        &[-self.last_size.w as f32, 0.0, 0.0].into()
                    );
                    base.enable_update(true);
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



