extern crate nalgebra as na;

mod tree;

pub use tree::{Events, Tree, Leaf, Base};

pub mod controls {
    use super::*;

    pub struct Text;

    impl Element for Text {
        fn inflate(&mut self, mut base: Base) {
            base.enable_update(true);
        }

        fn resize(&mut self, mut _base: Base, size: ElementSize) -> Option<ResolvedSize> {
            match size {
                ElementSize::Auto => Some(ResolvedSize { w: 100, h: 60 }),
                ElementSize::Fixed { w, h } => Some(ResolvedSize { w, h }),
            }
        }
    }

    pub struct Button;

    impl Element for Button {
        fn inflate(&mut self, mut base: Base) {
            base.add(Text);
            base.add(Text);
        }

        fn resize(&mut self, mut base: Base, size: ElementSize) -> Option<ResolvedSize> {
            base.layout_vertical(size, 5)
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
            base.add(Button);
        }

        fn resize(&mut self, mut base: Base, size: ElementSize) -> Option<ResolvedSize> {
            base.layout_vertical(size, 5)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ElementSize {
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
    fn resize(&mut self, _base: Base, _size: ElementSize) -> Option<ResolvedSize>;

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