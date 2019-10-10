use super::buffers::{Buffers, LinePoint};
use failure;
use floating_duration::TimeAsFloat;
use gl;
use crate::na;
use crate::data;
use crate::ColorBuffer;
use crate::Program;
use resources::Resources;
use std::collections::VecDeque;
use std::time::Instant;

const FRAME_DATA_CAPACITY: usize = 20;

#[derive(Clone)]
struct FrameData {
    start: Instant,
    position: usize,
    items: [Item; FRAME_DATA_CAPACITY],
}

impl FrameData {
    pub fn new(start: Instant) -> FrameData {
        FrameData {
            start,
            position: 0,
            items: [Item {
                time: Instant::now(),
                color: (0.0, 0.0, 0.0, 0.0).into(),
            }; FRAME_DATA_CAPACITY],
        }
    }

    pub fn restart(&mut self, start: Instant) {
        self.start = start;
        self.position = 0;
    }

    pub fn push(&mut self, item: Item) {
        if self.position >= FRAME_DATA_CAPACITY {
            return;
        }

        self.items[self.position] = item;
        self.position += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().take(self.position)
    }
}

#[derive(Copy, Clone)]
pub struct Item {
    time: Instant,
    color: data::u2_u10_u10_u10_rev_float,
}

pub struct FrameProfiler {
    program: Program,
    program_view_projection_location: Option<i32>,
    buffers: Option<Buffers>,
    draw_enabled: bool,
    frame_data: FrameData,
    frame_data_history: VecDeque<FrameData>,
    frame_data_pool: Vec<FrameData>,
    view_width_pixels: i32,
    view_height_pixels: i32,
    bottom_offset_px: i32,
}

impl FrameProfiler {
    pub fn new(
        gl: &gl::Gl,
        res: &Resources,
        bottom_offset_px: i32,
    ) -> Result<FrameProfiler, failure::Error> {
        let program = Program::from_res(gl, res, "shaders/render_gl/profiler_lines")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection");

        Ok(FrameProfiler {
            program,
            program_view_projection_location,
            buffers: None,
            draw_enabled: true,
            frame_data: FrameData::new(Instant::now()),
            frame_data_history: VecDeque::with_capacity(4000),
            frame_data_pool: vec![FrameData::new(Instant::now()); 4000], // 4000 pixels width should be enough for everybody
            view_width_pixels: 4000,
            view_height_pixels: 500,
            bottom_offset_px,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    pub fn begin(&mut self) {
        let mut data = self
            .frame_data_pool
            .pop()
            .unwrap_or_else(|| FrameData::new(Instant::now()));
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
        self.frame_data.push(Item {
            color: (color.x, color.y, color.z, 0.6).into(),
            time: Instant::now(),
        });
    }

    fn update_buffer(&mut self, gl: &gl::Gl) {
        let fps_bar_30 = 2;
        let fps_bar_60 = 2;
        let fps_bar_144 = 2;
        let all_data_len = self
            .frame_data_history
            .iter()
            .flat_map(|v| v.iter())
            .count()
            * 2
            + fps_bar_30
            + fps_bar_60
            + fps_bar_144;

        let recreate_buffer_capacity = match self.buffers {
            None => Some(4096),
            Some(ref buffers) if buffers.vertex_capacity < all_data_len => {
                Some(buffers.vertex_capacity * 2)
            }
            _ => None,
        };

        if let Some(new_capacity) = recreate_buffer_capacity {
            self.buffers = Some(Buffers::new(gl, new_capacity));
        }

        let p90 = self.view_height_pixels as f32 * 0.80;
        let p90_fps15_ms = 1000.0 / 30.0;
        let y_scale = p90 / p90_fps15_ms;

        let bottom_offset = self.bottom_offset_px as f32;

        if let Some(ref mut buffers) = self.buffers {
            if all_data_len > 0 {
                buffers.lines_vbo.bind();
                if let Some(mut buffer) = unsafe {
                    buffers
                        .lines_vbo
                        .map_buffer_range_write_invalidate::<LinePoint>(0, all_data_len)
                } {
                    for (index, frame) in self.frame_data_history.iter().enumerate() {
                        let mut previous_instant = frame.start;
                        for item in frame.iter() {
                            let item_start_diff =
                                (previous_instant - frame.start).as_fractional_millis() as f32;
                            let item_end_diff =
                                (item.time - frame.start).as_fractional_millis() as f32;

                            buffer.push(LinePoint {
                                pos: (index as f32, bottom_offset + item_start_diff * y_scale)
                                    .into(),
                                color: item.color,
                            });

                            buffer.push(LinePoint {
                                pos: (index as f32, bottom_offset + item_end_diff * y_scale).into(),
                                color: item.color,
                            });

                            previous_instant = item.time;
                        }
                    }

                    // 30 fps bar, red
                    let bar_height = bottom_offset + 33.333 * y_scale;

                    buffer.push(LinePoint {
                        pos: (0.0, bar_height).into(),
                        color: (1.0, 0.0, 0.0, 0.6).into(),
                    });

                    buffer.push(LinePoint {
                        pos: (self.view_width_pixels as f32, bar_height).into(),
                        color: (1.0, 0.0, 0.0, 0.3).into(),
                    });

                    // 60 fps bar, yellow
                    let bar_height = bottom_offset + 16.666 * y_scale;

                    buffer.push(LinePoint {
                        pos: (0.0, bar_height).into(),
                        color: (1.0, 1.0, 0.0, 0.6).into(),
                    });

                    buffer.push(LinePoint {
                        pos: (self.view_width_pixels as f32, bar_height).into(),
                        color: (1.0, 1.0, 0.0, 0.3).into(),
                    });

                    // 144 fps bar, green
                    let bar_height = bottom_offset + 6.9 * y_scale;

                    buffer.push(LinePoint {
                        pos: (0.0, bar_height).into(),
                        color: (0.0, 1.0, 0.0, 0.6).into(),
                    });

                    buffer.push(LinePoint {
                        pos: (self.view_width_pixels as f32, bar_height).into(),
                        color: (0.0, 1.0, 0.0, 0.3).into(),
                    });
                }
                buffers.lines_vbo.unbind();
            }

            buffers.vertex_count = all_data_len;
        }
    }

    pub fn render(
        &mut self,
        gl: &gl::Gl,
        target: &ColorBuffer,
        vp_matrix: &na::Matrix4<f32>,
        view_width_pixels: i32,
        view_height_pixels: i32,
    ) {
        self.view_width_pixels = view_width_pixels;
        self.view_height_pixels = view_height_pixels;

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

                        gl.DrawArrays(gl::LINES, 0, buffers.vertex_count as i32);

                        target.disable_blend(gl);
                    }
                }
            }
        }
    }
}
