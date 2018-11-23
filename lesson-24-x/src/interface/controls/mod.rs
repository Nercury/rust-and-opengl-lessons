use ui::*;
use na;

pub mod presentation;

pub struct Label {
    text: Vec<primitives::Text>,
    delta_acc: f32,
    delta: i32,
    size: i32,
    rotation: f32,
}

impl Label {
    pub fn new() -> Label {
        Label {
            text: Vec::new(),
            delta_acc: 0.0,
            delta: 50,
            size: 0,
            rotation: 0.0,
        }
    }
}

impl Label {
    fn update_text_size(&mut self) {
        for (i, text) in self.text.iter_mut().enumerate() {
            text.set_transform(
                &(na::convert::<_, na::Projective3<_>>(na::Translation3::new(i as f32 * 10.0, i as f32 * 10.0 + self.size as f32, 0.0)) // translation
                    * na::convert::<_, na::Projective3<_>>(na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), self.rotation)) // rot
                    * na::convert::<_, na::Projective3<_>>(na::Similarity3::new(na::zero(), na::zero(), 0.05))) // scale
            );
        }
    }
}

impl Element for Label {
    fn inflate(&mut self, base: &mut Base) {
        {
            let primitives = base.primitives();
            for i in 0..100 {
                if let Some(t) = primitives.text("Kaip faina", false, false, false, [0, 0, 0, 255].into()) {
                    self.text.push(t);
                }
            }
            self.update_text_size();
        }
        base.enable_update(true);
    }

    fn resize(&mut self, base: &mut Base) {
        let box_size = base.box_size();
        base.resolve_size(match box_size {
            BoxSize::Hidden => None,
            BoxSize::Auto => Some(ResolvedSize { w: 100, h: 60 }),
            BoxSize::Fixed { w, h } => Some(ResolvedSize { w, h }),
        })
    }

    fn update(&mut self, base: &mut Base, delta: f32) {
        self.delta_acc += delta;
        if self.delta_acc > 2.0 {
            let height = match base.box_size() {
                BoxSize::Hidden => 0,
                BoxSize::Auto => 60,
                BoxSize::Fixed { h, .. } => h,
            };

            if self.size + self.delta > height || self.size + self.delta < 0 {
                self.delta = -self.delta;
            }

            self.size += self.delta;
            self.delta_acc = 0.0;
        }

        self.rotation += ::std::f32::consts::PI * delta;
        self.update_text_size();
    }
}

pub struct Button {
    margin: i32,
    step: i32,
    delta_acc: f32,
}

impl Button {
    pub fn new() -> Button {
        Button {
            margin: 10,
            step: 1,
            delta_acc: 0.0,
        }
    }
}

impl Element for Button {
    fn inflate(&mut self, base: &mut Base) {
        base.add(Label::new());
        base.add(Label::new());
        base.enable_update(true);
    }

    fn resize(&mut self, base: &mut Base) {
        base.layout_vertical(self.margin, self.margin)
    }

    fn update(&mut self, base: &mut Base, delta: f32) {
        self.delta_acc += delta;
        if self.delta_acc > 0.05 {
            self.margin += self.step;
            if self.margin > 50 || self.margin < 1 {
                self.step = -self.step;
            }
            base.invalidate_size();
            self.delta_acc = 0.0;
        }
    }
}

pub struct Fill {
}

impl Fill {
    pub fn new() -> Fill {
        Fill { }
    }
}

impl Element for Fill {
    fn inflate(&mut self, base: &mut Base) {
        base.add(Button::new());
        base.add(Label::new());
    }

    fn resize(&mut self, base: &mut Base) {
        base.layout_horizontal(20, 0)
    }
}