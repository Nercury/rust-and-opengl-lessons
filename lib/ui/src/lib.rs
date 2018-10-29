mod tree;

pub use tree::{Events, Tree, Fill, Leaf};

pub mod controls {
    use super::*;

    pub struct Button {
        min_width: i32,
        min_height: i32,
    }

    impl Button {
        pub fn new() -> Button {
            Button {
                min_width: 50,
                min_height: 30,
            }
        }
    }

    impl Element for Button {
        fn resize(&mut self, size: ElementSize) {
//            match size {
//                ElementSize::Auto => Some(Effect::Resize {
//                    w: self.min_width,
//                    h: self.min_height
//                }),
//                ElementSize::Fixed { w, h } => Some(Effect::Render {
//                    w: if w > self.min_width { w } else { self.min_width },
//                    h: if h > self.min_height { h } else { self.min_height }
//                })
//            }.into_iter().collect()
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
    Add { id: Ix, size: Option<(i32, i32)> },
    Remove { id: Ix },
    Resize { id: Ix, size: Option<(i32, i32)> },
}

pub trait Element {

    fn resize(&mut self, size: ElementSize);

}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ix(u32);

impl Ix {
    fn inc(&mut self) -> Ix {
        let next_id = *self;
        self.0 += 1;
        next_id
    }
}