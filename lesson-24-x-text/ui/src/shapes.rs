use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use crate::na;
pub use self::shared::ShapeSlot;

#[derive(Clone)]
pub struct Shapes {
    shared: Rc<RefCell<shared::InnerShapes>>,
}

impl Shapes {
    pub fn new() -> Shapes {
        Shapes {
            shared: Rc::new(RefCell::new(shared::InnerShapes::new())),
        }
    }

    pub fn create_from_svg(&self, tree: &usvg::Tree, transform: Option<na::Projective3<f32>>) -> Shape {
        let slot = self.shared.borrow_mut().create_from_svg(tree, transform);

        Shape {
            _slot: slot,
            shared: self.shared.clone(),
        }
    }
}

pub struct Shape {
    _slot: shared::ShapeSlot,
    shared: Rc<RefCell<shared::InnerShapes>>,
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Shape")
            .field("slot", &self._slot)
            .finish()
    }
}

impl Shape {
    pub fn set_transform(&self, transform: Option<na::Projective3<f32>>) {
        self.shared.borrow_mut().set_shape_transform(self._slot, transform);
    }

    pub fn absolute_transform(&self, parent_absolute_transform: &na::Projective3<f32>) -> Option<na::Projective3<f32>> {
        let shared = self.shared.borrow_mut();
        shared.get_shape_transform(self._slot).map(|bt| parent_absolute_transform * bt)
    }

    pub fn slot(&self) -> shared::ShapeSlot {
        self._slot
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        let mut shared = self.shared.borrow_mut();
        shared.inc_count(self._slot);

        Shape {
            _slot: self._slot,
            shared: self.shared.clone(),
        }
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        self.shared.borrow_mut().dec_count(self._slot);
    }
}

mod shared {
    use crate::na;
    use slotmap;
    use lyon_path::PathEvent;
    use crate::svg;
    use usvg;
    use usvg::Color;

    use std::f64::NAN;

    pub const FALLBACK_COLOR: Color = Color {
        red: 0,
        green: 0,
        blue: 0,
    };

    new_key_type! { pub struct ShapeSlot; }

    #[derive(Copy, Clone)]
    pub struct ShapeKeyData {
        count: usize,
    }

    pub struct ShapeItem {
        entries: Vec<PathEvent>,
        transform_idx: usize,
        shape_type: ShapeType,
        color: Color,
        opacity: f32,
    }

    pub enum ShapeType {
        Fill,
        Stroke,
    }

    pub struct ShapeValueData {
        transforms: Vec<na::Matrix3<f32>>,
        items: Vec<ShapeItem>,
        transform: Option<na::Projective3<f32>>,
    }

    pub struct InnerShapes {
        keys: slotmap::SlotMap<ShapeSlot, ShapeKeyData>,
        keys_values: slotmap::SecondaryMap<ShapeSlot, ShapeValueData>,
    }

    impl InnerShapes {
        pub fn new() -> InnerShapes {
            InnerShapes {
                keys: slotmap::SlotMap::with_key(),
                keys_values: slotmap::SecondaryMap::default(),
            }
        }

        pub fn inc_count(&mut self, slot: ShapeSlot) {
            self.keys[slot].count += 1;
        }

        pub fn dec_count(&mut self, slot: ShapeSlot) {
            let remove = match self.keys.get_mut(slot) {
                None => return,
                Some(ref mut data) => {
                    data.count -= 1;
                    data.count == 0
                }
            };

            if remove {
                self.keys_values.remove(slot);
                self.keys.remove(slot);
            }
        }

        pub fn set_shape_transform(&mut self, slot: ShapeSlot, transform: Option<na::Projective3<f32>>) {
            self.keys_values[slot].transform = transform;
        }

        pub fn get_shape_transform(&self, slot: ShapeSlot) -> Option<na::Projective3<f32>> {
            self.keys_values[slot].transform
        }

        pub fn create_from_svg(&mut self, rtree: &usvg::Tree, transform: Option<na::Projective3<f32>>) -> ShapeSlot {
            let mut transforms = Vec::new();
            let mut items = Vec::new();

            let mut prev_transform = usvg::Transform {
                a: NAN, b: NAN,
                c: NAN, d: NAN,
                e: NAN, f: NAN,
            };
            let _view_box = rtree.svg_node().view_box;

            for node in rtree.root().descendants() {
                use usvg::NodeExt;

                if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                    let t = node.transform();
                    if t != prev_transform {
                        transforms.push(na::Matrix3::new(
                            t.a as f32, t.c as f32, t.e as f32,
                            t.b as f32, t.d as f32, t.f as f32,
                            0.0, 0.0, 1.0
                        ));
                    }
                    prev_transform = t;

                    let transform_idx = transforms.len() as u32 - 1;

                    if let Some(ref fill) = p.fill {
                        // fall back to always use color fill
                        // no gradients (yet?)
                        let color = match fill.paint {
                            usvg::Paint::Color(c) => c,
                            _ => FALLBACK_COLOR,
                        };

                        items.push(ShapeItem {
                            entries: svg::convert_path(p).path_iter().collect(),
                            shape_type: ShapeType::Fill,
                            transform_idx: transform_idx as usize,
                            color,
                            opacity: fill.opacity.value() as f32,
                        });
                    }

                    if let Some(ref stroke) = p.stroke {
                        let (stroke_color, _stroke_opts) = svg::convert_stroke(stroke, &FALLBACK_COLOR);
                        items.push(ShapeItem {
                            entries: svg::convert_path(p).path_iter().collect(),
                            shape_type: ShapeType::Stroke,
                            transform_idx: transform_idx as usize,
                            color: stroke_color,
                            opacity: stroke.opacity.value() as f32,
                        });
                    }
                }
            }

            let value = ShapeValueData {
                transforms,
                items,
                transform,
            };

            let slot = self.keys.insert(ShapeKeyData { count: 1 });
            self.keys_values.insert(slot, value);

            slot
        }
    }
}