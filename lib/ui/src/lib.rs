mod tree;

pub use tree::{Events, Tree, Leaf, Children};

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
        fn resize(&mut self, _size: ElementSize, _children: Children) -> Option<ResolvedSize> {
            unimplemented!("button")
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
        fn resize(&mut self, _size: ElementSize, _children: Children) -> Option<ResolvedSize> {
            unimplemented!("fill")
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

pub enum ResizeDecision {
    AutoFromChildrenVertical,
}

pub trait Element {

    fn resize(&mut self, size: ElementSize, children: Children) -> Option<ResolvedSize>;

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