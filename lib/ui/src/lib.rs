extern crate nalgebra as na;

mod tree;

pub use tree::{Events, Tree, Leaf, Base};

pub mod controls {
    use super::*;

    pub struct Text;

    impl Element for Text {
        fn inflate(&mut self, _base: Base) {

        }

        fn resize(&mut self, mut _base: Base, size: BoxSize) -> Option<ResolvedSize> {
            match size {
                BoxSize::Hidden => None,
                BoxSize::Auto => Some(ResolvedSize { w: 100, h: 60 }),
                BoxSize::Fixed { w, h } => Some(ResolvedSize { w, h }),
            }
        }

        fn update(&mut self, mut _base: Base, _delta: f32) {
            println!("update Text");
        }
    }

    pub struct Button {
        margin: i32,
        step: i32,
        delta_acc: f32,
        last_size: Option<BoxSize>,
    }

    impl Button {
        pub fn new() -> Button {
            Button {
                margin:10,
                step: 1,
                delta_acc: 0.0,
                last_size: None,
            }
        }
    }

    impl Element for Button {
        fn inflate(&mut self, mut base: Base) {
            base.add(Text);
            base.add(Text);
            base.enable_update(true);
        }

        fn resize(&mut self, mut base: Base, size: BoxSize) -> Option<ResolvedSize> {
            self.last_size = Some(size);
            base.layout_vertical(size, self.margin)
        }

        fn update(&mut self, mut base: Base, delta: f32) {
            self.delta_acc += delta;
            if self.delta_acc > 0.05 {
                self.margin += self.step;
                if self.margin > 20 || self.margin < 1 {
                    self.step = -self.step;
                }

                if let Some(BoxSize::Fixed { w, h }) = self.last_size {
                    base.layout_vertical(BoxSize::Fixed { w: w, h: h }, self.margin);
                } else {
                    base.layout_vertical(BoxSize::Auto, self.margin);
                }

                self.delta_acc = 0.0;
            }
        }
    }

    pub struct Fill {
        fixed_size: Option<(i32, i32)>,
    }

    impl Fill {
        pub fn new() -> Fill {
            Fill {
                fixed_size: None,
            }
        }
    }

    impl Element for Fill {
        fn inflate(&mut self, mut base: Base) {
            base.add(Text);
            base.add(Button::new());
        }

        fn resize(&mut self, mut base: Base, size: BoxSize) -> Option<ResolvedSize> {
            base.layout_vertical(size, 5)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BoxSize {
    Hidden,
    Auto,
    Fixed { w: i32, h: i32 }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ResolvedSize {
    w: i32, h: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum Effect {
    Add { id: Ix, parent_id: Option<Ix> },
    Remove { id: Ix },
    Resize { id: Ix, size: Option<(i32, i32)> },
    Transform { id: Ix, absolute_transform: na::Projective3<f32> },
}

pub trait Element {

    fn inflate(&mut self, _base: Base) {}
    fn resize(&mut self, _base: Base, _size: BoxSize) -> Option<ResolvedSize>;
    fn update(&mut self, _base: Base, _delta: f32) {}

}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ix(u32);

impl Ix {
    fn inc(&mut self) -> Ix {
        let next_id = *self;
        self.0 += 1;
        next_id
    }
}