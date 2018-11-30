use std::iter;
use std::slice;

use lyon_geom::math::Point;
use lyon_path::PathEvent;
use lyon_path::iterator::PathIter;

use usvg::{Path, PathSegment};

fn point(x: f64, y: f64) -> Point {
    Point::new(x as f32, y as f32)
}

// Map usvg::PathSegment to lyon::path::PathEvent
fn as_event(ps: &PathSegment) -> PathEvent {
    match *ps {
        PathSegment::MoveTo { x, y } => PathEvent::MoveTo(point(x, y)),
        PathSegment::LineTo { x, y } => PathEvent::LineTo(point(x, y)),
        PathSegment::CurveTo { x1, y1, x2, y2, x, y, } => {
            PathEvent::CubicTo(point(x1, y1), point(x2, y2), point(x, y))
        }
        PathSegment::ClosePath => PathEvent::Close,
    }
}

pub struct PathConv<'a>(SegmentIter<'a>);

// Alias for the iterator returned by usvg::Path::iter()
type SegmentIter<'a> = slice::Iter<'a, PathSegment>;

// Alias for our `interface` iterator
type PathConvIter<'a> = iter::Map<SegmentIter<'a>, fn(&PathSegment) -> PathEvent>;

// Provide a function which gives back a PathIter which is compatible with
// tesselators, so we don't have to implement the PathIterator trait
impl<'a> PathConv<'a> {
    pub fn path_iter(self) -> PathIter<PathConvIter<'a>> {
        PathIter::new(self.0.map(as_event))
    }
}

pub fn convert_path(p: &Path) -> PathConv {
    PathConv(p.segments.iter())
}


use usvg::{self, Color, Paint, Stroke};
use lyon_tessellation::{self as tessellation, StrokeOptions};

pub fn convert_stroke(s: &Stroke, fallback_color: &Color) -> (Color, StrokeOptions) {
    let color = match s.paint {
        Paint::Color(c) => c,
        _ => *fallback_color,
    };
    let linecap = match s.linecap {
        usvg::LineCap::Butt => tessellation::LineCap::Butt,
        usvg::LineCap::Square => tessellation::LineCap::Square,
        usvg::LineCap::Round => tessellation::LineCap::Round,
    };
    let linejoin = match s.linejoin {
        usvg::LineJoin::Miter => tessellation::LineJoin::Miter,
        usvg::LineJoin::Bevel => tessellation::LineJoin::Bevel,
        usvg::LineJoin::Round => tessellation::LineJoin::Round,
    };

    let opt = StrokeOptions::tolerance(0.01)
        .with_line_width(s.width as f32)
        .with_line_cap(linecap)
        .with_line_join(linejoin);

    (color, opt)
}