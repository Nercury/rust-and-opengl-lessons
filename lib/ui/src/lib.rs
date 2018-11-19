#![forbid(unsafe_code)]

extern crate nalgebra as na;
#[macro_use] extern crate log;
#[macro_use] extern crate slotmap;
extern crate slab;
extern crate metrohash;
extern crate int_hash;
extern crate sha1;
extern crate byteorder;
extern crate font_kit;
extern crate harfbuzz_rs;
extern crate lyon_path;

mod tree;
pub mod primitives;
mod queues;
mod fonts;

pub use primitives::Primitives;
pub use tree::{Base, Events, LastResolvedSize, Leaf, Tree};
pub use fonts::{Fonts, Font, BufferRef, GlyphPosition, HintingOptions};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BoxSize {
    Hidden,
    Auto,
    Fixed { w: i32, h: i32 },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ResolvedSize {
    pub w: i32,
    pub h: i32,
}

impl ResolvedSize {
    pub fn from_flow(flow: FlowDirection, width: i32, forward_val: i32) -> ResolvedSize {
        match flow {
            FlowDirection::Horizontal => ResolvedSize { w: forward_val, h: width },
            FlowDirection::Vertical => ResolvedSize { w: width, h: forward_val },
        }
    }

    pub fn to_flow(&self, flow: FlowDirection) -> (i32, i32) {
        match flow {
            FlowDirection::Horizontal => (self.h, self.w),
            FlowDirection::Vertical => (self.w, self.h),
        }
    }

    pub fn par(&self, flow: FlowDirection) -> i32 {
        match flow { FlowDirection::Vertical => self.h, FlowDirection::Horizontal => self.w }
    }

    pub fn ort(&self, flow: FlowDirection) -> i32 {
        match flow { FlowDirection::Vertical => self.w, FlowDirection::Horizontal => self.h }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FlowDirection {
    Horizontal,
    Vertical
}

#[derive(Debug, Copy, Clone)]
pub enum Effect {
    Add {
        id: Ix,
        parent_id: Option<Ix>,
    },
    Remove {
        id: Ix,
    },
    Resize {
        id: Ix,
        size: Option<(i32, i32)>,
    },
    Transform {
        id: Ix,
        absolute_transform: Option<na::Projective3<f32>>,
    },
    TextAdd {
        buffer: fonts::BufferRef,
    },
    TextTransform {
        buffer_id: usize,
        absolute_transform: Option<na::Projective3<f32>>,
    },
    TextRemove {
        buffer_id: usize,
    }
}

pub trait Element {
    fn inflate(&mut self, _base: &mut Base) {}
    fn resize(&mut self, base: &mut Base) {
        base.layout_vertical(5)
    }
    fn update(&mut self, _base: &mut Base, _delta: f32) {}
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
