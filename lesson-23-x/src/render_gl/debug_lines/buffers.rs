use render_gl::data;
use nalgebra as na;
use gl;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct LinePoint {
    #[location = "0"]
    pub pos: data::f32_f32_f32,
    #[location = "1"]
    pub color: data::u2_u10_u10_u10_rev_float,
}

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Instance {
    #[location = "2"]
    #[divisor = "1"]
    pub model_m0: data::f32_f32_f32,
    #[location = "3"]
    #[divisor = "1"]
    pub model_m1: data::f32_f32_f32,
    #[location = "4"]
    #[divisor = "1"]
    pub model_m2: data::f32_f32_f32,
}

use render_gl::buffer::{Buffer, VertexArray};

pub struct MultiDrawItem {
    pub model_matrix: na::Matrix4<f32>,
    pub starting_index: i32,
    pub index_count: i32,
}

pub struct Buffers {
    pub vbo_capacity: usize,
    pub multi_draw_items: Vec<MultiDrawItem>,
    lines_vbo: Buffer,
    lines_instances_vbo: Buffer,
    lines_ebo: Buffer,
    pub lines_vao: VertexArray,
}

impl Buffers {
    pub fn new(gl: &gl::Gl, vbo_capacity: usize) -> Buffers {
        let lines_vbo = Buffer::new_array(&gl);
        let lines_instances_vbo = Buffer::new_array(&gl);
        let lines_ebo = Buffer::new_element_array(&gl);

        let lines_vao = VertexArray::new(gl);
        lines_vao.bind();
        lines_ebo.bind();

        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();

//        lines_instances_vbo.bind();
//        Instance::vertex_attrib_pointers(gl);
//        lines_instances_vbo.unbind();

        lines_vao.unbind();

        // resize vbo buffer

        lines_vbo.bind();
        lines_vbo.stream_draw_data_null::<LinePoint>(vbo_capacity);

        // resize index buffer and upload indices

        lines_ebo.bind();
        lines_ebo.stream_draw_data_null::<u32>(vbo_capacity);
        if let Some(mut buffer) = unsafe { lines_ebo.map_buffer_range_write_invalidate::<u32>(0, vbo_capacity) } {
            for i in 0..vbo_capacity {
                buffer[i] = i as u32;
            }
        }

        lines_vbo.unbind();
        lines_ebo.unbind();

        Buffers {
            vbo_capacity,
            lines_vbo,
            lines_instances_vbo,
            lines_ebo,
            multi_draw_items: Vec::new(),
            lines_vao,
        }
    }

    pub fn upload_vertices(&self, items: impl Iterator<Item = LinePoint>) {
        self.lines_vbo.bind();
        if let Some(mut buffer) = unsafe { self.lines_vbo.map_buffer_range_write_invalidate::<LinePoint>(0, self.vbo_capacity) } {
            for (index, item) in items.enumerate().take(self.vbo_capacity) {
                *unsafe { buffer.get_unchecked_mut(index) } = item;
            }
        }
        self.lines_vbo.unbind();
    }
}