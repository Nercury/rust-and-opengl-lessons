use failure;
use gl;
use nalgebra as na;
use crate::render_gl::buffer;
use crate::render_gl::ColorBuffer;
use crate::render_gl::Program;
use crate::resources::Resources;

use std::cell::RefCell;
use std::rc::Rc;

mod line_point;
mod shared_debug_lines;
use self::line_point::LinePoint;
use self::shared_debug_lines::SharedDebugLines;

#[derive(Copy, Clone)]
struct PolylineBuilderItem {
    point: na::Vector3<f32>,
    color: na::Vector4<f32>,
}

pub struct PolylineBuilder {
    items: Vec<PolylineBuilderItem>,
    containers: Rc<RefCell<SharedDebugLines>>,
}

impl PolylineBuilder {
    pub fn with_point(mut self, point: na::Vector3<f32>, color: na::Vector4<f32>) -> Self {
        self.items.push(PolylineBuilderItem { point, color });
        self
    }

    pub fn close_and_finish(self) -> Polyline {
        let first_item = self.items[0].clone();
        self.with_point(first_item.point, first_item.color).finish()
    }

    pub fn finish(self) -> Polyline {
        let points_len = self.items.len();
        let mut pairwise_points = Vec::with_capacity(points_len * 2 - 2);
        for (i, item) in self.items.into_iter().enumerate() {
            let mapped_point = LinePoint {
                pos: (item.point.x, item.point.y, item.point.z).into(),
                color: (item.color.x, item.color.y, item.color.z, item.color.w).into(),
            };
            if i == 0 || i == points_len - 1 {
                pairwise_points.push(mapped_point);
            } else {
                pairwise_points.push(mapped_point.clone());
                pairwise_points.push(mapped_point);
            }
        }

        let new_id = self.containers.borrow_mut().new_container(pairwise_points);

        Polyline {
            id: new_id,
            containers: self.containers,
        }
    }
}

pub struct Polyline {
    containers: Rc<RefCell<SharedDebugLines>>,
    id: i32,
}

impl Drop for Polyline {
    fn drop(&mut self) {
        self.containers.borrow_mut().remove_container(self.id);
    }
}

pub struct DebugLines {
    program: Program,
    containers: Rc<RefCell<SharedDebugLines>>,
    line_point_count: i32,
    lines_vbo: buffer::ArrayBuffer,
    lines_vao: buffer::VertexArray,
}

impl DebugLines {
    pub fn new(gl: &gl::Gl, res: &Resources) -> Result<DebugLines, failure::Error> {
        let lines_vbo = buffer::ArrayBuffer::new(&gl);
        let lines_vao = buffer::VertexArray::new(gl);
        lines_vao.bind();
        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();
        lines_vao.unbind();

        Ok(DebugLines {
            program: Program::from_res(gl, res, "shaders/render_gl/debug_lines")?,
            containers: Rc::new(RefCell::new(SharedDebugLines::new())),
            line_point_count: 0,
            lines_vbo,
            lines_vao,
        })
    }

    fn check_if_invalidated_and_reinitialize(&mut self) {
        let mut shared_debug_lines = self.containers.borrow_mut();

        if shared_debug_lines.invalidated {
            let all_data: Vec<LinePoint> = shared_debug_lines
                .containers
                .values()
                .flat_map(|v| v.iter().cloned())
                .collect();

            self.lines_vbo.bind();
            self.lines_vbo.static_draw_data(&all_data);
            self.lines_vbo.unbind();

            self.line_point_count = all_data.len() as i32;

            shared_debug_lines.invalidated = false;
        }
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer) {
        self.check_if_invalidated_and_reinitialize();

        if self.line_point_count > 0 {
            self.program.set_used();
            self.lines_vao.bind();

            unsafe {
                target.set_default_blend_func(gl);
                target.enable_blend(gl);

                gl.DrawArrays(
                    gl::LINES,             // mode
                    0,                     // starting index in the enabled arrays
                    self.line_point_count, // number of indices to be rendered
                );

                target.disable_blend(gl);
            }
        }
    }

    pub fn start_polyline(
        &self,
        start_point: na::Vector3<f32>,
        start_color: na::Vector4<f32>,
    ) -> PolylineBuilder {
        PolylineBuilder {
            items: vec![PolylineBuilderItem {
                point: start_point,
                color: start_color,
            }],
            containers: self.containers.clone(),
        }
    }
}
