use std::collections::VecDeque;
use nalgebra as na;
use failure;
use gl;
use resources::Resources;
use render_gl::Program;
use render_gl::ColorBuffer;
use render_gl::data;
use super::buffers::{LinePoint, Buffers};

const FRAME_DATA_CAPACITY: usize = 5;

#[derive(Clone)]
struct FrameData {
    position: usize,
    items: [Item; FRAME_DATA_CAPACITY],
}

impl FrameData {
    pub fn new() -> FrameData {
        FrameData {
            position: 0,
            items: [Item { count: 0, color: (0.0,0.0,0.0,0.0,).into() }; FRAME_DATA_CAPACITY],
        }
    }

    pub fn restart(&mut self) {
        self.position = 0;
    }

    pub fn push(&mut self, item: Item) {
        if self.position >= FRAME_DATA_CAPACITY { return; }

        self.items[self.position] = item;
        self.position += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().take(self.position)
    }
}

#[derive(Copy, Clone)]
pub struct Item {
    count: usize,
    color: data::u2_u10_u10_u10_rev_float,
}

pub struct EventCountProfiler {
    program: Program,
    program_view_projection_location: Option<i32>,
    buffers: Option<Buffers>,
    draw_enabled: bool,
    frame_data: FrameData,
    frame_data_history: VecDeque<FrameData>,
    frame_data_pool: Vec<FrameData>,
    view_width_pixels: i32,
    single_item_height: i32,
    bottom_offset_px: i32,
}

impl EventCountProfiler {
    pub fn new(gl: &gl::Gl, res: &Resources, single_item_height: i32, bottom_offset_px: i32) -> Result<EventCountProfiler, failure::Error> {
        let program = Program::from_res(gl, res, "shaders/render_gl/profiler_lines")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection");

        Ok(EventCountProfiler {
            program,
            program_view_projection_location,
            buffers: None,
            draw_enabled: true,
            frame_data: FrameData::new(),
            frame_data_history: VecDeque::with_capacity(4000),
            frame_data_pool: vec![FrameData::new(); 4000],
            view_width_pixels: 500,
            single_item_height,
            bottom_offset_px,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    pub fn begin(&mut self) {
        let mut data = self.frame_data_pool.pop().unwrap_or_else(|| {
            FrameData::new()
        });
        data.restart();

        ::std::mem::swap(&mut data, &mut self.frame_data);

        self.frame_data_history.push_front(data);

        while self.frame_data_history.len() as i32 > self.view_width_pixels {
            if let Some(old) = self.frame_data_history.pop_back() {
                self.frame_data_pool.push(old);
            }
        }
    }

    pub fn push(&mut self, value: usize, color: na::Vector3<f32>) {
        self.frame_data.push(Item {
            color: (color.x, color.y, color.z, 0.6).into(),
            count: value,
        });
    }

    fn update_buffer(&mut self, gl: &gl::Gl) {
        let all_data_len = self.frame_data_history
            .iter()
            .flat_map(|v| v.iter())
            .count() * 2;

        let recreate_buffer_capacity = match self.buffers {
            None => Some(4096),
            Some(ref buffers) if buffers.vertex_capacity < all_data_len => Some(buffers.vertex_capacity * 2),
            _ => None,
        };

        if let Some(new_capacity) = recreate_buffer_capacity {
            self.buffers = Some(Buffers::new(gl, new_capacity));
        }

        let y_scale = self.single_item_height as f32;
        let bottom_offset = self.bottom_offset_px as f32;

        if let Some(ref mut buffers) = self.buffers {
            if all_data_len > 0 {
                buffers.lines_vbo.bind();
                if let Some(mut buffer) = unsafe { buffers.lines_vbo.map_buffer_range_write_invalidate::<LinePoint>(0, all_data_len) } {
                    for (index, frame) in self.frame_data_history.iter().enumerate() {
                        let mut sum: f32 = 0.0;
                        for item in frame.iter() {
                            let item_start_diff = sum;
                            sum += item.count as f32;
                            let item_end_diff = sum;

                            buffer.push(LinePoint {
                                pos: (index as f32, bottom_offset + item_start_diff * y_scale).into(),
                                color: item.color,
                            });

                            buffer.push(LinePoint {
                                pos: (index as f32, bottom_offset + item_end_diff * y_scale).into(),
                                color: item.color,
                            });
                        }
                    }
                }
                buffers.lines_vbo.unbind();
            }

            buffers.vertex_count = all_data_len;
        }
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>, view_width_pixels: i32) {
        self.view_width_pixels = view_width_pixels;

        if self.draw_enabled {
            self.update_buffer(gl);

            if let Some(ref buffers) = self.buffers {
                if buffers.vertex_count > 0 {
                    self.program.set_used();
                    if let Some(loc) = self.program_view_projection_location {
                        self.program.set_uniform_matrix_4fv(loc, &vp_matrix);
                    }

                    buffers.lines_vao.bind();

                    unsafe {
                        target.set_default_blend_func(gl);
                        target.enable_blend(gl);

                        gl.DrawArrays(
                            gl::LINES,
                            0,
                            buffers.vertex_count as i32,
                        );

                        target.disable_blend(gl);
                    }
                }
            }
        }
    }
}