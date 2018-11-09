use failure;
use gl;
use nalgebra as na;
use ncollide3d;
use render_gl::buffer;
use render_gl::data;
use render_gl::ColorBuffer;
use render_gl::Program;
use resources::Resources;

use std::cell::RefCell;
use std::rc::Rc;

mod line_point;
mod shared_debug_lines;

use self::line_point::LinePoint;
use self::shared_debug_lines::SharedDebugLines;

struct MultiDrawItem {
    model_matrix: na::Matrix4<f32>,
    starting_index: i32,
    index_count: i32,
}

pub struct DebugLines {
    program: Program,
    program_view_projection_location: Option<i32>,
    program_model_matrix_location: Option<i32>,
    containers: Rc<RefCell<SharedDebugLines>>,
    multi_draw_items: Vec<MultiDrawItem>,
    lines_vbo: buffer::ArrayBuffer,
    lines_vbo_capacity: Option<usize>,
    lines_vao: buffer::VertexArray,
    draw_enabled: bool,
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
        let program_model_matrix_location = program.get_uniform_location("Model");

        Ok(DebugLines {
            program,
            program_view_projection_location,
            program_model_matrix_location,
            containers: Rc::new(RefCell::new(SharedDebugLines::new())),
            lines_vbo,
            multi_draw_items: Vec::new(),
            lines_vbo_capacity: None,
            lines_vao,
            draw_enabled: true,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    fn check_if_invalidated_and_reinitialize(&mut self) {
        let mut shared_debug_lines = self.containers.borrow_mut();

        if shared_debug_lines.invalidated {
            let all_data_len = shared_debug_lines
                .containers
                .values()
                .flat_map(|v| v.data.iter())
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
                        .flat_map(|v| v.data.iter())
                        .enumerate()
                    {
                        *unsafe { buffer.get_unchecked_mut(index) } = *item;
                    }

                    self.multi_draw_items.clear();
                    let mut offset = 0;
                    for container in shared_debug_lines.containers.values() {
                        self.multi_draw_items.push(MultiDrawItem {
                            model_matrix: container.isometry.to_homogeneous(),
                            starting_index: offset,
                            index_count: container.data.len() as i32,
                        });
                        offset += container.data.len() as i32;
                    }
                }
            }
            self.lines_vbo.unbind();

            shared_debug_lines.invalidated = false;
        }
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        if self.draw_enabled {
            self.check_if_invalidated_and_reinitialize();

            if self.multi_draw_items.len() > 0 {
                self.program.set_used();
                if let Some(loc) = self.program_view_projection_location {
                    self.program.set_uniform_matrix_4fv(loc, &vp_matrix);
                }

                let program_model_matrix_location = self
                    .program_model_matrix_location
                    .expect("Debug lines Model uniform must exist");

                self.lines_vao.bind();

                unsafe {
                    target.set_default_blend_func(gl);
                    target.enable_blend(gl);

                    for instance in &self.multi_draw_items {
                        self.program.set_uniform_matrix_4fv(
                            program_model_matrix_location,
                            &instance.model_matrix,
                        );

                        gl.DrawArrays(
                            gl::LINES, // mode
                            instance.starting_index,
                            instance.index_count,
                        );
                    }

                    target.disable_blend(gl);
                }
            }
        }
    }

    pub fn marker(&self, pos: na::Point3<f32>, size: f32) -> PointMarker {
        let half = size / 2.0;

        let new_id = self.containers.borrow_mut().new_container(
            na::Isometry3::from_parts(
                na::Translation3::from_vector(pos.coords),
                na::UnitQuaternion::identity(),
            ),
            vec![
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
            ],
        );

        PointMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }

    pub fn colored_marker(
        &self,
        pos: na::Point3<f32>,
        color: na::Vector4<f32>,
        size: f32,
    ) -> PointMarker {
        let half = size / 2.0;

        let new_id = self.containers.borrow_mut().new_container(
            na::Isometry3::from_parts(
                na::Translation3::from_vector(pos.coords),
                na::UnitQuaternion::identity(),
            ),
            vec![
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
            ],
        );

        PointMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }

    pub fn ray_marker(
        &self,
        isometry: na::Isometry3<f32>,
        pos: na::Point3<f32>,
        direction: na::Vector3<f32>,
        color: na::Vector4<f32>,
    ) -> RayMarker {
        let end = pos + direction;
        let end_color = na::Vector4::new(color.x, color.y, color.z, 0.0);

        let new_id = self.containers.borrow_mut().new_container(
            isometry,
            vec![
                LinePoint {
                    pos: render_p3(pos),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3(end),
                    color: render_color_vec4(end_color),
                },
            ],
        );

        RayMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }

    pub fn aabb_marker(
        &self,
        isometry: na::Isometry3<f32>,
        aabb: ncollide3d::bounding_volume::aabb::AABB<f32>,
        color: na::Vector4<f32>,
    ) -> AabbMarker {
        let a = aabb.mins();
        let b = aabb.maxs();

        let new_id = self.containers.borrow_mut().new_container(
            isometry,
            vec![
                LinePoint {
                    pos: render_p3([a.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, b.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([a.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, b.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, a.z].into()),
                    color: render_color_vec4(color),
                },
                LinePoint {
                    pos: render_p3([b.x, a.y, b.z].into()),
                    color: render_color_vec4(color),
                },
            ],
        );

        AabbMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }

    pub fn grid_marker(
        &self,
        isometry: na::Isometry3<f32>,
        spacing: f32,
        count: i32,
        color: na::Vector4<f32>,
    ) -> GridMarker {
        let mut lines = Vec::new();

        let mut half_count = count / 2;
        if half_count == 0 {
            half_count = 1;
        }

        for x in -half_count..=half_count {
            let start = na::Point3::new(x as f32 * spacing, -half_count as f32 * spacing, 0.0);
            let end = na::Point3::new(x as f32 * spacing, half_count as f32 * spacing, 0.0);

            lines.push(LinePoint {
                pos: render_p3(start),
                color: render_color_vec4(color),
            });
            lines.push(LinePoint {
                pos: render_p3(end),
                color: render_color_vec4(color),
            });
        }

        for y in -half_count..=half_count {
            let start = na::Point3::new(-half_count as f32 * spacing, y as f32 * spacing, 0.0);
            let end = na::Point3::new(half_count as f32 * spacing, y as f32 * spacing, 0.0);

            lines.push(LinePoint {
                pos: render_p3(start),
                color: render_color_vec4(color),
            });
            lines.push(LinePoint {
                pos: render_p3(end),
                color: render_color_vec4(color),
            });
        }

        let new_id = self.containers.borrow_mut().new_container(isometry, lines);

        GridMarker {
            containers: self.containers.clone(),
            id: new_id,
        }
    }
}

pub struct AabbMarker {
    containers: Rc<RefCell<SharedDebugLines>>,
    pub id: i32,
}

impl AabbMarker {
    pub fn update_isometry(&self, isometry: na::Isometry3<f32>) {
        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data.isometry = isometry;
        }
    }
}

impl Drop for AabbMarker {
    fn drop(&mut self) {
        self.containers.borrow_mut().remove_container(self.id);
    }
}

pub struct GridMarker {
    containers: Rc<RefCell<SharedDebugLines>>,
    pub id: i32,
}

impl GridMarker {
    pub fn update_isometry(&self, isometry: na::Isometry3<f32>) {
        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data.isometry = isometry;
        }
    }
}

impl Drop for GridMarker {
    fn drop(&mut self) {
        self.containers.borrow_mut().remove_container(self.id);
    }
}

pub struct RayMarker {
    containers: Rc<RefCell<SharedDebugLines>>,
    id: i32,
}

impl RayMarker {
    pub fn update_ray_pos_and_dir(&self, pos: na::Point3<f32>, direction: na::Vector3<f32>) {
        let end = pos + direction;

        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data.data[0].pos = render_p3(pos);
            data.data[1].pos = render_p3(end);
        }
    }

    pub fn update_isometry(&self, isometry: na::Isometry3<f32>) {
        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data.isometry = isometry;
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
}

impl PointMarker {
    pub fn update_position(&self, pos: na::Point3<f32>) {
        if let Some(data) = self.containers.borrow_mut().get_container_mut(self.id) {
            data.isometry = na::Isometry3::from_parts(
                na::Translation3::from_vector(pos.coords),
                na::UnitQuaternion::identity(),
            );
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
