use failure;
use gl;
use nalgebra as na;
use crate::render_gl::buffer;
use crate::render_gl::data;
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
                pairwise_points.push(mapped_point);
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
    program_view_projection_location: Option<i32>,
    containers: Rc<RefCell<SharedDebugLines>>,
    lines_vbo_count: i32,
    lines_vbo: buffer::ArrayBuffer,
    lines_vbo_capacity: Option<usize>,
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

        let program = Program::from_res(gl, res, "shaders/render_gl/debug_lines")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection");

        Ok(DebugLines {
            program,
            program_view_projection_location,
            containers: Rc::new(RefCell::new(SharedDebugLines::new())),
            lines_vbo,
            lines_vbo_count: 0,
            lines_vbo_capacity: None,
            lines_vao,
        })
    }

    fn check_if_invalidated_and_reinitialize(&mut self) {
        let mut shared_debug_lines = self.containers.borrow_mut();

        if shared_debug_lines.invalidated {
            let all_data_len = shared_debug_lines
                .containers
                .values()
                .flat_map(|v| v.iter())
                .count();

            self.lines_vbo.bind();

            let should_recreate_buffer = match self.lines_vbo_capacity {
                None => true,
                Some(lines_vbo_capacity) if lines_vbo_capacity < all_data_len => true,
                _ => false,
            };

            if should_recreate_buffer {
                self.lines_vbo
                    .dynamic_draw_data_null::<LinePoint>(all_data_len);
                self.lines_vbo_capacity = Some(all_data_len);
            }

            if let Some(_) = self.lines_vbo_capacity {
                if let Some(mut buffer) = unsafe {
                    self.lines_vbo
                        .map_buffer_range_write_invalidate::<LinePoint>(0, all_data_len)
                } {
                    for (index, item) in shared_debug_lines
                        .containers
                        .values()
                        .flat_map(|v| v.iter())
                        .enumerate()
                    {
                        *unsafe { buffer.get_unchecked_mut(index) } = *item;
                    }
                }
            }
            self.lines_vbo.unbind();

            self.lines_vbo_count = all_data_len as i32;

            shared_debug_lines.invalidated = false;
        }
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        self.check_if_invalidated_and_reinitialize();

        if self.lines_vbo_count > 0 {
            self.program.set_used();
            if let Some(loc) = self.program_view_projection_location {
                self.program.set_uniform_matrix_4fv(loc, &vp_matrix);
            }

            self.lines_vao.bind();

            unsafe {
                target.set_default_blend_func(gl);
                target.enable_blend(gl);

                gl.DrawArrays(
                    gl::LINES,            // mode
                    0,                    // starting index in the enabled arrays
                    self.lines_vbo_count, // number of indices to be rendered
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

    pub fn marker(&self, pos: na::Point3<f32>, size: f32) -> PointMarker {
        let half = size / 2.0;

        let new_id = self.containers.borrow_mut().new_container(vec![
            LinePoint {
                pos: render_p3(pos + na::Vector3::x() * half),
                color: (0.0, 1.0, 0.0, 1.0).into(),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::x() * -half),
                color: (0.0, 1.0, 0.0, 1.0).into(),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::y() * half),
                color: (1.0, 0.0, 0.0, 1.0).into(),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::y() * -half),
                color: (1.0, 0.0, 0.0, 1.0).into(),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::z() * half),
                color: (0.0, 0.0, 1.0, 1.0).into(),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::z() * -half),
                color: (0.0, 0.0, 1.0, 1.0).into(),
            },
        ]);

        PointMarker {
            containers: self.containers.clone(),
            id: new_id,
            half_size: half,
        }
    }

    pub fn colored_marker(
        &self,
        pos: na::Point3<f32>,
        color: na::Vector4<f32>,
        size: f32,
    ) -> PointMarker {
        let half = size / 2.0;

        let new_id = self.containers.borrow_mut().new_container(vec![
            LinePoint {
                pos: render_p3(pos + na::Vector3::x() * half),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::x() * -half),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::y() * half),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::y() * -half),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::z() * half),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(pos + na::Vector3::z() * -half),
                color: render_color_vec4(color),
            },
        ]);

        PointMarker {
            containers: self.containers.clone(),
            id: new_id,
            half_size: half,
        }
    }

    pub fn ray_marker(
        &self,
        pos: na::Point3<f32>,
        direction: na::Vector3<f32>,
        color: na::Vector4<f32>,
    ) -> RayMarker {
        let end = pos + direction;
        let end_color = na::Vector4::new(color.x, color.y, color.z, 0.0);

        let new_id = self.containers.borrow_mut().new_container(vec![
            LinePoint {
                pos: render_p3(pos),
                color: render_color_vec4(color),
            },
            LinePoint {
                pos: render_p3(end),
                color: render_color_vec4(end_color),
            },
        ]);

        RayMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }
}

pub struct RayMarker {
    containers: Rc<RefCell<SharedDebugLines>>,
    id: i32,
}

impl RayMarker {
    pub fn update_ray(&self, pos: na::Point3<f32>, direction: na::Vector3<f32>) {
        let end = pos + direction;

        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data[0].pos = render_p3(pos);
            data[1].pos = render_p3(end);
        }
    }
}

impl Drop for RayMarker {
    fn drop(&mut self) {
        self.containers.borrow_mut().remove_container(self.id);
    }
}

pub struct PointMarker {
    containers: Rc<RefCell<SharedDebugLines>>,
    id: i32,
    half_size: f32,
}

impl PointMarker {
    pub fn update_position(&self, pos: na::Point3<f32>) {
        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            let half = self.half_size;

            data[0].pos = render_p3(pos + na::Vector3::x() * half);
            data[1].pos = render_p3(pos + na::Vector3::x() * -half);

            data[2].pos = render_p3(pos + na::Vector3::y() * half);
            data[3].pos = render_p3(pos + na::Vector3::y() * -half);

            data[4].pos = render_p3(pos + na::Vector3::z() * half);
            data[5].pos = render_p3(pos + na::Vector3::z() * -half);
        }
    }
}

impl Drop for PointMarker {
    fn drop(&mut self) {
        self.containers.borrow_mut().remove_container(self.id);
    }
}

fn render_p3(v: na::Point3<f32>) -> data::f32_f32_f32 {
    data::f32_f32_f32::new(v.x, v.y, v.z)
}

fn render_color_vec4(v: na::Vector4<f32>) -> data::u2_u10_u10_u10_rev_float {
    (v.x, v.y, v.z, v.w).into()
}
