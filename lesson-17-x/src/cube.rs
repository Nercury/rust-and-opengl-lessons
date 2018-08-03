use gl;
use failure;
use render_gl::{self, data, buffer};
use resources::Resources;
use nalgebra as na;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::u2_u10_u10_u10_rev_float,
    #[location = "2"]
    normal: data::f32_f32_f32,
    #[location = "3"]
    uv: data::f16_f16,
}

pub struct Cube {
    program: render_gl::Program,
    texture: render_gl::Texture,
    texture_specular: render_gl::Texture,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    camera_pos_location: Option<i32>,
    tex_face_location: Option<i32>,
    tex_specular_location: Option<i32>,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
    _debug_rays: Vec<render_gl::RayMarker>,
}

impl Cube {
    pub fn new(res: &Resources, gl: &gl::Gl, debug_lines: &render_gl::DebugLines) -> Result<Cube, failure::Error> {

        // set up shader program

//        let texture = render_gl::Texture::from_res_rgb("textures/dice_high_contrast.png").load(gl, res)?;
        let texture = render_gl::Texture::from_res_rgb("textures/dice.png").load(gl, res)?;
        let texture_specular = render_gl::Texture::from_res_rgb("textures/dice_specular.png").load(gl, res)?;

        let program = render_gl::Program::from_res(gl, res, "shaders/cube")?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let camera_pos_location = program.get_uniform_location("CameraPos");
        let tex_face_location = program.get_uniform_location("TexFace");
        let tex_specular_location = program.get_uniform_location("TexSpecular");

        // ----------- A: stupid mapping

//        let v0 = (-1.0, -1.0, -1.0);
//        let v1 = (1.0,  -1.0, -1.0);
//        let v2 = (-1.0,  1.0, -1.0);
//        let v3 = (1.0,  1.0, -1.0);
//        let v4 = (-1.0, -1.0, 1.0);
//        let v5 = (1.0,  -1.0, 1.0);
//        let v6 = (-1.0,  1.0, 1.0);
//        let v7 = (1.0,  1.0, 1.0);
//
//        let vbo_data = vec![
//            Vertex { pos: v0.into(), clr: (1.0, 0.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv: (0.0, 0.0).into() }, // 0
//            Vertex { pos: v1.into(), clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv: (1.0, 0.0).into() }, // 1
//            Vertex { pos: v2.into(), clr: (0.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv: (0.0, 1.0).into() }, // 2
//            Vertex { pos: v3.into(),  clr: (1.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv: (1.0, 1.0).into() }, // 3
//
//            Vertex { pos: v4.into(),  clr: (0.0, 0.3, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (0.0, 0.0).into() }, // 4
//            Vertex { pos: v5.into(),  clr: (1.0, 0.0, 0.3, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (1.0, 0.0).into() }, // 5
//            Vertex { pos: v6.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (0.0, 1.0).into() }, // 6
//            Vertex { pos: v7.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (1.0, 1.0).into() }, // 7
//
//            Vertex { pos: v0.into(),  clr: (0.0, 1.0, 0.3, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (0.0, 0.0).into() }, // 8
//            Vertex { pos: v1.into(),  clr: (1.0, 0.0, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (1.0, 0.0).into() }, // 9
//            Vertex { pos: v4.into(),  clr: (0.5, 0.7, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (0.0, 1.0).into() }, // 10
//            Vertex { pos: v5.into(),  clr: (1.0, 0.5, 0.1, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (1.0, 1.0).into() }, // 11
//
//            Vertex { pos: v2.into(),  clr: (0.3, 1.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (0.0, 0.0).into() }, // 12
//            Vertex { pos: v3.into(),  clr: (0.8, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (1.0, 0.0).into() }, // 13
//            Vertex { pos: v6.into(),  clr: (0.5, 0.5, 0.4, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (0.0, 1.0).into() }, // 14
//            Vertex { pos: v7.into(),  clr: (0.4, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (1.0, 1.0).into() }, // 15
//
//            Vertex { pos: v0.into(),  clr: (0.0, 0.4, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (0.0, 0.0).into() }, // 16
//            Vertex { pos: v2.into(),  clr: (1.0, 0.0, 0.4, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (1.0, 0.0).into() }, // 17
//            Vertex { pos: v4.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (0.0, 1.0).into() }, // 18
//            Vertex { pos: v6.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (1.0, 1.0).into() }, // 19
//
//            Vertex { pos: v1.into(),  clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (0.0, 0.0).into() }, // 20
//            Vertex { pos: v3.into(),  clr: (0.1, 0.0, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (1.0, 0.0).into() }, // 21
//            Vertex { pos: v5.into(),  clr: (0.1, 0.7, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (0.0, 1.0).into() }, // 22
//            Vertex { pos: v7.into(),  clr: (1.0, 0.1, 0.7, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (1.0, 1.0).into() }, // 23
//        ];

        // ------------- B: Better mapping

        let v0 = (-1.0, -1.0, -1.0);
        let v1 = (1.0,  -1.0, -1.0);
        let v2 = (-1.0,  1.0, -1.0);
        let v3 = (1.0,  1.0, -1.0);
        let v4 = (-1.0, -1.0, 1.0);
        let v5 = (1.0,  -1.0, 1.0);
        let v6 = (-1.0,  1.0, 1.0);
        let v7 = (1.0,  1.0, 1.0);

        let a = 16.0 / 1024.0;
        let b = 336.0 / 1024.0;
        let c = 352.0 / 1024.0;
        let d = 672.0 / 1024.0;
        let e = 688.0 / 1024.0;
        let f = 1008.0 / 1024.0;

        let vbo_data = vec![
            // 6
            Vertex { pos: v0.into(), clr: (1.0, 0.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv:  (e, c).into() }, // 0
            Vertex { pos: v1.into(), clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv:  (f, c).into() }, // 1
            Vertex { pos: v2.into(), clr: (0.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv:  (e, d).into() }, // 2
            Vertex { pos: v3.into(),  clr: (1.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into(), uv: (f, d).into() }, // 3

            // 1
            Vertex { pos: v4.into(),  clr: (0.0, 0.3, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (a, b).into() }, // 4
            Vertex { pos: v5.into(),  clr: (1.0, 0.0, 0.3, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (b, b).into() }, // 5
            Vertex { pos: v6.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (a, a).into() }, // 6
            Vertex { pos: v7.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: (b, a).into() }, // 7

            // 2
            Vertex { pos: v0.into(),  clr: (0.0, 1.0, 0.3, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (c, b).into() }, // 8
            Vertex { pos: v1.into(),  clr: (1.0, 0.0, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (d, b).into() }, // 9
            Vertex { pos: v4.into(),  clr: (0.5, 0.7, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (c, a).into() }, // 10
            Vertex { pos: v5.into(),  clr: (1.0, 0.5, 0.1, 1.0).into(), normal: (0.0, -1.0, 0.0).into(), uv: (d, a).into() }, // 11

            // 4
            Vertex { pos: v2.into(),  clr: (0.3, 1.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (a, c).into() }, // 12
            Vertex { pos: v3.into(),  clr: (0.8, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (b, c).into() }, // 13
            Vertex { pos: v6.into(),  clr: (0.5, 0.5, 0.4, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (a, d).into() }, // 14
            Vertex { pos: v7.into(),  clr: (0.4, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into(), uv: (b, d).into() }, // 15

            // 3
            Vertex { pos: v0.into(),  clr: (0.0, 0.4, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (f, b).into() }, // 16
            Vertex { pos: v2.into(),  clr: (1.0, 0.0, 0.4, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (e, b).into() }, // 17
            Vertex { pos: v4.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (f, a).into() }, // 18
            Vertex { pos: v6.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (-1.0, 0.0, 0.0).into(), uv: (e, a).into() }, // 19

            // 5
            Vertex { pos: v1.into(),  clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (d, c).into() }, // 20
            Vertex { pos: v3.into(),  clr: (0.1, 0.0, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (c, c).into() }, // 21
            Vertex { pos: v5.into(),  clr: (0.1, 0.7, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (d, d).into() }, // 22
            Vertex { pos: v7.into(),  clr: (1.0, 0.1, 0.7, 1.0).into(), normal: (1.0, 0.0, 0.0).into(), uv: (c, d).into() }, // 23
        ];

        let ebo_data: Vec<u8> = vec![
            0,2,1,
            1,2,3,

            4,5,6,
            6,5,7,

            8,11,10,
            8,9,11,

            12,14,15,
            12,15,13,

            16,18,17,
            18,19,17,

            20,21,22,
            22,21,23,
        ];

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vbo_data);
        vbo.unbind();

        let ebo = buffer::ElementArrayBuffer::new(gl);
        ebo.bind();
        ebo.static_draw_data(&ebo_data);
        ebo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        ebo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind();

        vbo.unbind();
        ebo.unbind();

        Ok(Cube {
            texture,
            texture_specular,
            program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            tex_face_location,
            tex_specular_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
            _debug_rays: vbo_data.iter().map(|v| debug_lines.ray_marker(
                na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                na::Vector3::new(v.normal.d0, v.normal.d1, v.normal.d2),
                na::Vector4::new(v.clr.inner.x(), v.clr.inner.y(), v.clr.inner.z(), v.clr.inner.w())
            )).collect()
        })
    }

    pub fn render(&self, gl: &gl::Gl, view_matrix: &na::Matrix4<f32>, proj_matrix: &na::Matrix4<f32>, camera_pos: &na::Vector3<f32>) {
        self.program.set_used();

        if let Some(loc) = self.tex_face_location {
            self.texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let Some(loc) = self.tex_specular_location {
            self.texture_specular.bind_at(2);
            self.program.set_uniform_1i(loc, 2);
        }

        if let Some(loc) = self.program_view_location {
            self.program.set_uniform_matrix_4fv(loc, view_matrix);
        }
        if let Some(loc) = self.program_projection_location {
            self.program.set_uniform_matrix_4fv(loc, proj_matrix);
        }
        if let Some(loc) = self.camera_pos_location {
            self.program.set_uniform_3f(loc, camera_pos);
        }
        self.vao.bind();

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES, // mode
                self.index_count, // index vertex count
                gl::UNSIGNED_BYTE, // index type
                ::std::ptr::null() // pointer to indices (we are using ebo configured at vao creation)
            );
        }
    }
}