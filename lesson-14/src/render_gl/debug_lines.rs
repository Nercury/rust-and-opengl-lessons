use render_gl::data;
use render_gl::Program;
use render_gl::buffer;
use resources::Resources;
use gl;
use failure;

#[derive(VertexAttribPointers)]
#[repr(C, packed)]
struct LinePoint {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    color: data::u2_u10_u10_u10_rev_float,
}

pub struct DebugLines {
    program: Program,
    line_point_count: i32,
    lines_vbo: buffer::ArrayBuffer,
    lines_vao: buffer::VertexArray,
}

impl DebugLines {
    pub fn new(gl: &gl::Gl, res: &Resources) -> Result<DebugLines, failure::Error> {
        let line_points = vec![
            LinePoint {
                pos: (0.5, -0.5, 0.0).into(),
                color: (1.0, 0.0, 0.0, 1.0).into(),
            },
            LinePoint {
                pos: (0.5, 0.5, 0.0).into(),
                color: (0.0, 1.0, 0.0, 1.0).into(),
            }
        ];

        let lines_vbo = buffer::ArrayBuffer::new(&gl);
        lines_vbo.bind();
        lines_vbo.static_draw_data(&line_points);
        lines_vbo.unbind();

        let lines_vao = buffer::VertexArray::new(gl);
        lines_vao.bind();
        lines_vbo.bind();
        LinePoint::vertex_attrib_pointers(gl);
        lines_vbo.unbind();
        lines_vao.unbind();

        Ok(DebugLines {
            program: Program::from_res(gl, res, "render_gl/debug_lines")?,
            line_point_count: line_points.len() as i32,
            lines_vbo,
            lines_vao,
        })
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.program.set_used();
        self.lines_vao.bind();

        unsafe {
            gl.DrawArrays(
                gl::LINES, // mode
                0, // starting index in the enabled arrays
                self.line_point_count // number of indices to be rendered
            );
        }
    }
}