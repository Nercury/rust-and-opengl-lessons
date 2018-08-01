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
}

pub struct Cube {
    program: render_gl::Program,
    program_view_projection_location: i32,
    camera_pos_location: i32,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
    _debug_rays: Vec<render_gl::RayMarker>,
}

impl Cube {
    pub fn new(res: &Resources, gl: &gl::Gl, debug_lines: &render_gl::DebugLines) -> Result<Cube, failure::Error> {

        // set up shader program

        let program = render_gl::Program::from_res(gl, res, "shaders/cube")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection")?;
        let camera_pos_location = program.get_uniform_location("CameraPos")?;

        let v0 = (-1.0, -1.0, -1.0);
        let v1 = (1.0,  -1.0, -1.0);
        let v2 = (-1.0,  1.0, -1.0);
        let v3 = (1.0,  1.0, -1.0);
        let v4 = (-1.0, -1.0, 1.0);
        let v5 = (1.0,  -1.0, 1.0);
        let v6 = (-1.0,  1.0, 1.0);
        let v7 = (1.0,  1.0, 1.0);

        // ----------- part A, shared normals

//        let vbo_data = vec![
//            Vertex { pos: v0.into(), clr: (1.0, 0.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 0
//            Vertex { pos: v1.into(), clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 1
//            Vertex { pos: v2.into(), clr: (0.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 2
//            Vertex { pos: v3.into(),  clr: (1.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 3
//
//            Vertex { pos: v4.into(),  clr: (0.0, 1.0, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 4
//            Vertex { pos: v5.into(),  clr: (1.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 5
//            Vertex { pos: v6.into(),  clr: (0.5, 0.5, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 6
//            Vertex { pos: v7.into(),   clr: (1.0, 0.5, 0.5, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 7
//        ];
//
//        let ebo_data: Vec<u8> = vec![
//            0, 1, 5,
//            0, 5, 4,
//            1, 3, 7,
//            1, 7, 5,
//            4, 5, 7,
//            4, 7, 6,
//            2, 6, 7,
//            2, 7, 3,
//            0, 4, 6,
//            0, 6, 2,
//            0, 2, 3,
//            0, 3, 1,
//        ];

        // ---------- part B, properly shared normals

//        let mut vbo_data = vec![
//            Vertex { pos: v0.into(), clr: (1.0, 0.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 0
//            Vertex { pos: v1.into(), clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 1
//            Vertex { pos: v2.into(), clr: (0.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 2
//            Vertex { pos: v3.into(),  clr: (1.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 3
//
//            Vertex { pos: v4.into(),  clr: (0.0, 1.0, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 4
//            Vertex { pos: v5.into(),  clr: (1.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 5
//            Vertex { pos: v6.into(),  clr: (0.5, 0.5, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 6
//            Vertex { pos: v7.into(),  clr: (1.0, 0.5, 0.5, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 7
//        ];
//        for Vertex { pos, ref mut normal, .. } in vbo_data.iter_mut() {
//            let n = (na::Point3::new(pos.d0, pos.d1, pos.d2) - na::Point3::origin()).normalize();
//            normal.d0 = n.x;
//            normal.d1 = n.y;
//            normal.d2 = n.z;
//        }
//
//        let ebo_data: Vec<u8> = vec![
//            0, 1, 5,
//            0, 5, 4,
//            1, 3, 7,
//            1, 7, 5,
//            4, 5, 7,
//            4, 7, 6,
//            2, 6, 7,
//            2, 7, 3,
//            0, 4, 6,
//            0, 6, 2,
//            0, 2, 3,
//            0, 3, 1,
//        ];

        // ----------- part C, separate normals

        let vbo_data = vec![
            Vertex { pos: v0.into(), clr: (1.0, 0.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 0
            Vertex { pos: v1.into(), clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 1
            Vertex { pos: v2.into(), clr: (0.0, 0.0, 1.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 2
            Vertex { pos: v3.into(),  clr: (1.0, 1.0, 0.0, 1.0).into(), normal: (0.0, 0.0, -1.0).into() }, // 3

            Vertex { pos: v4.into(),  clr: (0.0, 0.3, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 4
            Vertex { pos: v5.into(),  clr: (1.0, 0.0, 0.3, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 5
            Vertex { pos: v6.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 6
            Vertex { pos: v7.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (0.0, 0.0, 1.0).into() }, // 7

            Vertex { pos: v0.into(),  clr: (0.0, 1.0, 0.3, 1.0).into(), normal: (0.0, -1.0, 0.0).into() }, // 8
            Vertex { pos: v1.into(),  clr: (1.0, 0.0, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into() }, // 9
            Vertex { pos: v4.into(),  clr: (0.5, 0.7, 1.0, 1.0).into(), normal: (0.0, -1.0, 0.0).into() }, // 10
            Vertex { pos: v5.into(),  clr: (1.0, 0.5, 0.1, 1.0).into(), normal: (0.0, -1.0, 0.0).into() }, // 11

            Vertex { pos: v2.into(),  clr: (0.3, 1.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into() }, // 12
            Vertex { pos: v3.into(),  clr: (0.8, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into() }, // 13
            Vertex { pos: v6.into(),  clr: (0.5, 0.5, 0.4, 1.0).into(), normal: (0.0, 1.0, 0.0).into() }, // 14
            Vertex { pos: v7.into(),  clr: (0.4, 0.0, 1.0, 1.0).into(), normal: (0.0, 1.0, 0.0).into() }, // 15

            Vertex { pos: v0.into(),  clr: (0.0, 0.4, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into() }, // 16
            Vertex { pos: v2.into(),  clr: (1.0, 0.0, 0.4, 1.0).into(), normal: (-1.0, 0.0, 0.0).into() }, // 17
            Vertex { pos: v4.into(),  clr: (0.7, 0.5, 1.0, 1.0).into(), normal: (-1.0, 0.0, 0.0).into() }, // 18
            Vertex { pos: v6.into(),  clr: (1.0, 0.7, 0.5, 1.0).into(), normal: (-1.0, 0.0, 0.0).into() }, // 19

            Vertex { pos: v1.into(),  clr: (0.0, 1.0, 0.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into() }, // 20
            Vertex { pos: v3.into(),  clr: (0.1, 0.0, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into() }, // 21
            Vertex { pos: v5.into(),  clr: (0.1, 0.7, 1.0, 1.0).into(), normal: (1.0, 0.0, 0.0).into() }, // 22
            Vertex { pos: v7.into(),  clr: (1.0, 0.1, 0.7, 1.0).into(), normal: (1.0, 0.0, 0.0).into() }, // 23
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
        ebo.static_draw_data::<u8>(&ebo_data);
        ebo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        ebo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind();

        ebo.unbind(); // do not unbind ebo until we finish with vao

        Ok(Cube {
            program,
            program_view_projection_location,
            camera_pos_location,
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

    pub fn render(&self, gl: &gl::Gl, vp_matrix: &na::Matrix4<f32>, camera_pos: &na::Vector3<f32>) {
        self.program.set_used();
        self.program.set_uniform_matrix4fv(self.program_view_projection_location, vp_matrix);
        self.program.set_uniform_3f(self.camera_pos_location, camera_pos);
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