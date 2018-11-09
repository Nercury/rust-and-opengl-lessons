use gl;
use render_gl::buffer::{Buffer, VertexArray};
use render_gl::data;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct LinePoint {
    #[location = "0"]
    pub pos: data::f32_f32,
    #[location = "1"]
    pub color: data::u2_u10_u10_u10_rev_float,
}

pub struct Buffers {
    pub vertex_capacity: usize,
    pub vertex_count: usize,
    pub lines_vbo: Buffer,
    pub lines_vao: VertexArray,
}

impl Buffers {
    pub fn new(gl: &gl::Gl, vertex_capacity: usize) -> Buffers {
        let lines_vbo = Buffer::new_array(&gl);
        let lines_vao = VertexArray::new(gl);
        lines_vao.bind();
        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();
        lines_vao.unbind();

        if vertex_capacity > 0 {
            lines_vbo.bind();
            lines_vbo.stream_draw_data_null::<LinePoint>(vertex_capacity);
            lines_vbo.unbind();
        }

        Buffers {
            vertex_capacity,
            vertex_count: 0,
            lines_vbo,
            lines_vao,
        }
    }
}
