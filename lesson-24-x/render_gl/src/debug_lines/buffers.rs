use data;
use gl;
use na;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct LinePoint {
    #[location = "0"]
    pub pos: data::f32_f32_f32,
    #[location = "1"]
    pub color: data::u2_u10_u10_u10_rev_float,
}

use buffer::{Buffer, VertexArray};

pub struct MultiDrawItem {
    pub model_matrix: na::Matrix4<f32>,
    pub starting_index: i32,
    pub index_count: i32,
}

pub struct Buffers {
    pub vbo_capacity: usize,
    pub multi_draw_items: Vec<MultiDrawItem>,
    lines_vbo: Buffer,
    pub lines_vao: VertexArray,
}

impl Buffers {
    pub fn new(gl: &gl::Gl, vbo_capacity: usize) -> Buffers {
        let lines_vbo = Buffer::new_array(&gl);
        let lines_vao = VertexArray::new(gl);

        lines_vao.bind();

        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();

        lines_vao.unbind();

        if vbo_capacity > 0 {
            lines_vbo.bind();
            lines_vbo.stream_draw_data_null::<LinePoint>(vbo_capacity);
            lines_vbo.unbind();
        }

        Buffers {
            vbo_capacity,
            lines_vbo,
            multi_draw_items: Vec::new(),
            lines_vao,
        }
    }

    pub fn upload_vertices(&self, items: impl Iterator<Item = LinePoint>) {
        if self.vbo_capacity > 0 {
            self.lines_vbo.bind();
            if let Some(mut buffer) = unsafe {
                self.lines_vbo
                    .map_buffer_range_write_invalidate::<LinePoint>(0, self.vbo_capacity)
            } {
                for (index, item) in items.enumerate().take(self.vbo_capacity) {
                    *unsafe { buffer.get_unchecked_mut(index) } = item;
                }
            }
            self.lines_vbo.unbind();
        }
    }
}
