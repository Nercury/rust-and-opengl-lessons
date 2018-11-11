use data;
use buffer::Buffer;
use buffer::VertexArray;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderVertex {
    #[location = "0"]
    pub pos: data::f32_f32,
    #[location = "1"]
    pub normal: data::f32_f32,
}

pub struct Buffers {
    lines_vbo: Buffer,
    pub lines_vao: VertexArray,
}

impl Buffers {
    pub fn new(gl: &gl::Gl) -> Buffers {
        let lines_vbo = Buffer::new_array(&gl);
        let lines_vao = VertexArray::new(gl);

        lines_vao.bind();

        lines_vbo.bind();
        FlatlanderVertex::vertex_attrib_pointers(gl);
        lines_vbo.unbind();

        lines_vao.unbind();

        Buffers {
            lines_vbo,
            lines_vao,
        }
    }

    pub fn upload_vertices(&self, items: impl Iterator<Item = FlatlanderVertex>) {
//        if self.vbo_capacity > 0 {
//            self.lines_vbo.bind();
//            if let Some(mut buffer) = unsafe {
//                self.lines_vbo
//                    .map_buffer_range_write_invalidate::<Vertex>(0, self.vbo_capacity)
//            } {
//                for (index, item) in items.enumerate().take(self.vbo_capacity) {
//                    *unsafe { buffer.get_unchecked_mut(index) } = item;
//                }
//            }
//            self.lines_vbo.unbind();
//        }
    }
}