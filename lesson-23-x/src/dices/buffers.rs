use gl;
use mesh;
use render_gl::buffer::{Buffer, VertexArray};
use render_gl::data;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct ModelVertex {
    #[location = "0"]
    pub pos: data::f32_f32_f32,
    #[location = "1"]
    pub uv: data::f16_f16,
    #[location = "2"]
    pub t: data::f32_f32_f32,
    #[location = "3"]
    pub n: data::f32_f32_f32,
}

pub struct Buffers {
    _vbo: Buffer,
    _ebo: Buffer,
    pub vao: VertexArray,
    pub index_count: i32,
}

impl Buffers {
    pub fn new(gl: &gl::Gl, mesh: &mesh::Mesh) -> Buffers {
        let vbo_data = mesh
            .vertices
            .clone()
            .into_iter()
            .map(|v| {
                let tv = v.tangents.unwrap_or_else(|| {
                    println!("Missing tangent vectors");
                    mesh::Tangents::nans()
                });
                let uv = v.uv.unwrap_or_else(|| {
                    println!("Missing uv vectors");
                    [0.0, 0.0].into()
                });
                let normal = v.normal.unwrap_or_else(|| {
                    println!("Missing normal vectors");
                    [0.0, 0.0, 0.0].into()
                });
                ModelVertex {
                    pos: (v.pos.x, v.pos.y, v.pos.z).into(),
                    uv: (uv.x, -uv.y).into(),
                    t: (tv.tangent.x, tv.tangent.y, tv.tangent.z).into(),
                    n: (normal.x, normal.y, normal.z).into(),
                }
            }).collect::<Vec<_>>();

        let ebo_data = mesh.triangle_indices();

        let vbo = Buffer::new_array(gl);
        vbo.bind();
        vbo.stream_draw_data(&vbo_data);
        vbo.unbind();

        let ebo = Buffer::new_element_array(gl);
        ebo.bind();
        ebo.stream_draw_data(&ebo_data);
        ebo.unbind();

        // set up vertex array object

        let vao = VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        ebo.bind();
        ModelVertex::vertex_attrib_pointers(gl);
        vao.unbind();

        vbo.unbind();
        ebo.unbind();

        Buffers {
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
        }
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.vao.bind();

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES, // mode
                self.index_count, // index vertex count
                gl::UNSIGNED_INT, // index type
                ::std::ptr::null(), // pointer to indices (we are using ebo configured at vao creation)
            );
        }

        self.vao.unbind();
    }
}
