use std::collections::VecDeque;
use nalgebra as na;
use failure;
use gl;
use resources::Resources;
use render_gl::Program;
use render_gl::ColorBuffer;
use render_gl::buffer::{ArrayBuffer, VertexArray};
use render_gl::data;
use std::time::Instant;
use floating_duration::TimeAsFloat;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct LinePoint {
    #[location = "0"]
    pub pos: data::f32_f32,
    #[location = "1"]
    pub color: data::u2_u10_u10_u10_rev_float,
}

struct FrameData {
    start: Instant,
    items: Vec<Item>,
}

impl FrameData {
    pub fn restart(&mut self, start: Instant) {
        self.start = start;
        self.items.clear();
    }
}

impl FrameData {
    pub fn new(start: Instant) -> FrameData {
        FrameData {
            start,
            items: vec![]
        }
    }
}

struct Item {
    time: Instant,
    color: data::u2_u10_u10_u10_rev_float,
}

pub struct Profiler {
    program: Program,
    program_view_projection_location: Option<i32>,
    lines_vbo: ArrayBuffer,
    lines_vbo_capacity: Option<usize>,
    lines_vbo_count: i32,
    lines_vao: VertexArray,
    draw_enabled: bool,
    frame_data: FrameData,
    frame_data_history: VecDeque<FrameData>,
    frame_data_pool: Vec<FrameData>,
    view_width_pixels: i32,
}

impl Profiler {
    pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Profiler, failure::Error> {
        let program = Program::from_res(gl, res, "shaders/render_gl/profiler_lines")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection");

        let lines_vbo = ArrayBuffer::new(&gl);
        let lines_vao = VertexArray::new(gl);
        lines_vao.bind();
        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();
        lines_vao.unbind();

        Ok(Profiler {
            program,
            program_view_projection_location,
            lines_vbo,
            lines_vbo_capacity: None,
            lines_vbo_count: 0,
            lines_vao,
            draw_enabled: true,
            frame_data: FrameData::new(Instant::now()),
            frame_data_history: VecDeque::with_capacity(1000),
            frame_data_pool: Vec::with_capacity(1000),
            view_width_pixels: 500,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    pub fn begin(&mut self) {
        let mut data = self.frame_data_pool.pop().unwrap_or_else(|| FrameData::new(Instant::now()));
        data.restart(Instant::now());

        ::std::mem::swap(&mut data, &mut self.frame_data);

        self.frame_data_history.push_front(data);

        while self.frame_data_history.len() as i32 > self.view_width_pixels {
            if let Some(old) = self.frame_data_history.pop_back() {
                self.frame_data_pool.push(old);
            }
        }
    }

    pub fn push(&mut self, color: na::Vector3<f32>) {
        self.frame_data.items.push(Item {
            color: (color.x, color.y, color.z, 0.6).into(),
            time: Instant::now(),
        });
    }

    fn update_buffer(&mut self) {
        let fps_bar_30 = 2;
        let fps_bar_60 = 2;
        let fps_bar_144 = 2;
        let all_data_len = self.frame_data_history
            .iter()
            .flat_map(|v| v.items.iter())
            .count() * 2 + fps_bar_30 + fps_bar_60 + fps_bar_144;

        self.lines_vbo.bind();

        let should_recreate_buffer = match self.lines_vbo_capacity {
            None => true,
            Some(lines_vbo_capacity) if lines_vbo_capacity < all_data_len => true,
            _ => false,
        };

        if should_recreate_buffer {
            let old_capacity = self.lines_vbo_capacity.unwrap_or(0);
            let new_capacity = if old_capacity == 0 { 1024 } else { old_capacity * 2 };

            self.lines_vbo.dynamic_draw_data_null::<LinePoint>(new_capacity);
            self.lines_vbo_capacity = Some(new_capacity);
        }

        const Y_SCALE: f32 = 10.0;

        if let Some(_) = self.lines_vbo_capacity {
            if let Some(mut buffer) = unsafe { self.lines_vbo.map_buffer_range_write_invalidate::<LinePoint>(0, all_data_len) } {
                let mut buffer_index = 0;
                for (index, frame) in self.frame_data_history.iter().enumerate() {
                    let mut previous_instant = frame.start;
                    for item in frame.items.iter() {
                        let item_start_diff = (previous_instant - frame.start).as_fractional_millis() as f32;
                        let item_end_diff = (item.time - frame.start).as_fractional_millis() as f32;

                        let line_point = LinePoint {
                            pos: (index as f32, item_start_diff * Y_SCALE).into(),
                            color: item.color,
                        };
                        *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point;
                        buffer_index += 1;

                        let line_point = LinePoint {
                            pos: (index as f32, item_end_diff * Y_SCALE).into(),
                            color: item.color,
                        };
                        *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point;
                        buffer_index += 1;

                        previous_instant = item.time;
                    }
                }

                // 30 fps bar, red
                let bar_height = 33.333 * Y_SCALE;

                let line_point = LinePoint {
                    pos: (0.0, bar_height).into(),
                    color: (1.0, 0.0, 0.0, 0.6).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;

                let line_point = LinePoint {
                    pos: (self.view_width_pixels as f32, bar_height).into(),
                    color: (1.0, 0.0, 0.0, 0.3).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;

                // 60 fps bar, yellow
                let bar_height = 16.666 * Y_SCALE;

                let line_point = LinePoint {
                    pos: (0.0, bar_height).into(),
                    color: (1.0, 1.0, 0.0, 0.6).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;

                let line_point = LinePoint {
                    pos: (self.view_width_pixels as f32, bar_height).into(),
                    color: (1.0, 1.0, 0.0, 0.3).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;

                // 144 fps bar, green
                let bar_height = 6.9 * Y_SCALE;

                let line_point = LinePoint {
                    pos: (0.0, bar_height).into(),
                    color: (0.0, 1.0, 0.0, 0.6).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;

                let line_point = LinePoint {
                    pos: (self.view_width_pixels as f32, bar_height).into(),
                    color: (0.0, 1.0, 0.0, 0.3).into(),
                };
                *unsafe { buffer.get_unchecked_mut(buffer_index) } = line_point; buffer_index += 1;
            }
        }

        self.lines_vbo_count = (all_data_len + fps_bar_30 + fps_bar_60 + fps_bar_144) as i32;

        self.lines_vbo.unbind();
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>, view_width_pixels: i32) {
        self.view_width_pixels = view_width_pixels;
        if self.draw_enabled {
            self.update_buffer();

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
                        gl::LINES,
                        0,
                        self.lines_vbo_count,
                    );

                    target.disable_blend(gl);
                }
            }
        }
    }
}