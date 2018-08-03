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
    normal_tn: data::f32_f32_f32,
    #[location = "3"]
    normal_btn: data::f32_f32_f32,
    #[location = "4"]
    normal: data::f32_f32_f32,
    #[location = "5"]
    uv: data::f16_f16,
}

#[derive(Copy, Clone)]
struct V {
    pos: na::Vector3<f32>,
    clr: na::Vector4<f32>,
    uv: na::Vector2<f32>,
}

struct T(usize, usize, usize);

impl T {
    fn vertices(&self, vertices: &[V]) -> (V, V, V) {
        (vertices[self.0], vertices[self.1], vertices[self.2])
    }

    fn tangent_vectors(&self, &(ref a, ref b, ref c): &(V, V, V)) -> (na::Vector3<f32>, na::Vector3<f32>, na::Vector3<f32>) {
        let edge2 = c.pos - a.pos;
        let edge1 = b.pos - a.pos;

        let delta_uv1 = c.uv - a.uv;
        let delta_uv2 = b.uv - a.uv;

        let f = 1.0f32 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
        let tangent = na::Vector3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z)
        ).normalize();
        let bitangent = na::Vector3::new(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z)
        ).normalize();
        let tangent_normal = tangent.cross(&bitangent).normalize();

        (tangent, bitangent, tangent_normal)
    }

    fn normal(&self, &(ref a, ref b, ref c): &(V, V, V)) -> na::Vector3<f32> {
        let edge1 = c.pos - a.pos;
        let edge2 = b.pos - a.pos;

        edge1.cross(&edge2).normalize()
    }
}

pub struct Cube {
    program: render_gl::Program,
    texture: render_gl::Texture,
    texture_normal: render_gl::Texture,
    texture_specular: render_gl::Texture,
    program_view_location: i32,
    program_projection_location: i32,
    camera_pos_location: i32,
    tex_face_location: i32,
    tex_normal_location: i32,
    tex_specular_location: i32,
    _vbo: buffer::ArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
    _debug_rays: Vec<render_gl::RayMarker>,
}

impl Cube {
    pub fn new(res: &Resources, gl: &gl::Gl, debug_lines: &render_gl::DebugLines) -> Result<Cube, failure::Error> {

        // set up shader program

//        let texture = render_gl::Texture::from_res_rgb("textures/dice_high_contrast.png").load(gl, res)?;
        let texture = render_gl::Texture::from_res_rgb("textures/dice.png").load(gl, res)?;
//        let texture_normal = render_gl::Texture::from_res_rgb("textures/dice_normal.png").load(gl, res)?;
        let texture_normal = render_gl::Texture::from_res_rgb("textures/normal_mapping_normal_map.png").load(gl, res)?;
        let texture_normal = render_gl::Texture::from_res_rgb("textures/brickwall_normal.jpg").load(gl, res)?;
        let texture_specular = render_gl::Texture::from_res_rgb("textures/dice_specular.png").load(gl, res)?;
        let program = render_gl::Program::from_res(gl, res, "shaders/cube")?;
        let program_view_location = program.get_uniform_location("View")?;
        let program_projection_location = program.get_uniform_location("Projection")?;
        let camera_pos_location = program.get_uniform_location("CameraPos")?;
        let tex_face_location = program.get_uniform_location("TexFace")?;
        let tex_normal_location = program.get_uniform_location("TexNormal")?;
        let tex_specular_location = program.get_uniform_location("TexSpecular")?;

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
//
//        let ebo_data: Vec<u8> = vec![
//            0,2,1,
//            1,2,3,
//
//            4,5,6,
//            6,5,7,
//
//            8,11,10,
//            8,9,11,
//
//            12,14,15,
//            12,15,13,
//
//            16,18,17,
//            18,19,17,
//
//            20,21,22,
//            22,21,23,
//        ];

        // ----------- A: omg annoying correct mapping

        // http://cpetry.github.io/NormalMap-Online/

        let v0 = [-1.0, -1.0, -1.0];
        let v1 = [1.0,  -1.0, -1.0];
        let v2 = [-1.0,  1.0, -1.0];
        let v3 = [1.0,  1.0, -1.0];
        let v4 = [-1.0, -1.0, 1.0];
        let v5 = [1.0,  -1.0, 1.0];
        let v6 = [-1.0,  1.0, 1.0];
        let v7 = [1.0,  1.0, 1.0];

        let a = 16.0 / 1024.0;
        let b = 336.0 / 1024.0;
        let c = 352.0 / 1024.0;
        let d = 672.0 / 1024.0;
        let e = 688.0 / 1024.0;
        let f = 1008.0 / 1024.0;

        let vertices = vec![
            // 6
            V { pos: v0.into(), clr: [1.0, 0.0, 0.0, 1.0].into(), uv:  [e, c].into() }, // 0
            V { pos: v1.into(), clr: [0.0, 1.0, 0.0, 1.0].into(), uv:  [f, c].into() }, // 1
            V { pos: v2.into(), clr: [0.0, 0.0, 1.0, 1.0].into(), uv:  [e, d].into() }, // 2
            V { pos: v3.into(),  clr: [1.0, 1.0, 0.0, 1.0].into(), uv: [f, d].into() }, // 3

            // 1
            V { pos: v4.into(),  clr: [0.0, 0.3, 1.0, 1.0].into(), uv: [a, b].into() }, // 4
            V { pos: v5.into(),  clr: [1.0, 0.0, 0.3, 1.0].into(), uv: [b, b].into() }, // 5
            V { pos: v6.into(),  clr: [0.7, 0.5, 1.0, 1.0].into(), uv: [a, a].into() }, // 6
            V { pos: v7.into(),  clr: [1.0, 0.7, 0.5, 1.0].into(), uv: [b, a].into() }, // 7

            // 2
            V { pos: v0.into(),  clr: [0.0, 1.0, 0.3, 1.0].into(), uv: [c, b].into() }, // 8
            V { pos: v1.into(),  clr: [1.0, 0.0, 1.0, 1.0].into(), uv: [d, b].into() }, // 9
            V { pos: v4.into(),  clr: [0.5, 0.7, 1.0, 1.0].into(), uv: [c, a].into() }, // 10
            V { pos: v5.into(),  clr: [1.0, 0.5, 0.1, 1.0].into(), uv: [d, a].into() }, // 11

            // 4
            V { pos: v2.into(),  clr: [0.3, 1.0, 1.0, 1.0].into(), uv: [a, c].into() }, // 12
            V { pos: v3.into(),  clr: [0.8, 0.0, 1.0, 1.0].into(), uv: [b, c].into() }, // 13
            V { pos: v6.into(),  clr: [0.5, 0.5, 0.4, 1.0].into(), uv: [a, d].into() }, // 14
            V { pos: v7.into(),  clr: [0.4, 0.0, 1.0, 1.0].into(), uv: [b, d].into() }, // 15

            // 3
            V { pos: v2.into(),  clr: [0.0, 0.4, 1.0, 1.0].into(), uv: [f, b].into() }, // 16
            V { pos: v0.into(),  clr: [1.0, 0.0, 0.4, 1.0].into(), uv: [e, b].into() }, // 17
            V { pos: v6.into(),  clr: [0.7, 0.5, 1.0, 1.0].into(), uv: [f, a].into() }, // 18
            V { pos: v4.into(),  clr: [1.0, 0.7, 0.5, 1.0].into(), uv: [e, a].into() }, // 19

            // 5
            V { pos: v1.into(),  clr: [0.0, 1.0, 0.0, 1.0].into(), uv: [d, c].into() }, // 20
            V { pos: v3.into(),  clr: [0.1, 0.0, 1.0, 1.0].into(), uv: [c, c].into() }, // 21
            V { pos: v5.into(),  clr: [0.1, 0.7, 1.0, 1.0].into(), uv: [d, d].into() }, // 22
            V { pos: v7.into(),  clr: [1.0, 0.1, 0.7, 1.0].into(), uv: [c, d].into() }, // 23
        ];

        let triangles: Vec<T> = vec![
//            T(0,2,1),
//            T(1,2,3),
//
//            T(4,5,6),
//            T(6,5,7),
//
//            T(8,11,10),
//            T(8,9,11),
//
//            T(12,14,15),
//            T(12,15,13),

            T(16,17,18),
            T(18,17,19),

            T(20,21,22),
            T(22,21,23),
        ];

        let mut vbo_data = Vec::with_capacity(triangles.len() * 3);
        for (tangents, normal, t) in triangles.into_iter().map(|t| {
            let vertices = t.vertices(&vertices);
            (t.tangent_vectors(&vertices), t.normal(&vertices), vertices)
        }) {
            vbo_data.push(Vertex {
                pos: (t.0.pos.x, t.0.pos.y, t.0.pos.z).into(),
                normal_tn: (tangents.0.x, tangents.0.y, tangents.0.z).into(),
                normal_btn: (tangents.1.x, tangents.1.y, tangents.1.z).into(),
                normal: (tangents.2.x, tangents.2.y, tangents.2.z).into(),
                clr: (t.0.clr.x, t.0.clr.y, t.0.clr.z, t.0.clr.w).into(),
                uv: (t.0.uv.x, t.0.uv.y).into(),
            });
            vbo_data.push(Vertex {
                pos: (t.1.pos.x, t.1.pos.y, t.1.pos.z).into(),
                normal_tn: (tangents.0.x, tangents.0.y, tangents.0.z).into(),
                normal_btn: (tangents.1.x, tangents.1.y, tangents.1.z).into(),
                normal: (tangents.2.x, tangents.2.y, tangents.2.z).into(),
                clr: (t.1.clr.x, t.1.clr.y, t.1.clr.z, t.1.clr.w).into(),
                uv: (t.1.uv.x, t.1.uv.y).into(),
            });
            vbo_data.push(Vertex {
                pos: (t.2.pos.x, t.2.pos.y, t.2.pos.z).into(),
                normal_tn: (tangents.0.x, tangents.0.y, tangents.0.z).into(),
                normal_btn: (tangents.1.x, tangents.1.y, tangents.1.z).into(),
                normal: (tangents.2.x, tangents.2.y, tangents.2.z).into(),
                clr: (t.2.clr.x, t.2.clr.y, t.2.clr.z, t.2.clr.w).into(),
                uv: (t.2.uv.x, t.2.uv.y).into(),
            });
        }

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vbo_data);
        vbo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind();
        vbo.unbind();

        Ok(Cube {
            texture,
            texture_normal,
            texture_specular,
            program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            tex_face_location,
            tex_normal_location,
            tex_specular_location,
            _vbo: vbo,
            index_count: vbo_data.len() as i32,
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

        self.texture.bind_at(0);
        self.program.set_uniform_1i(self.tex_face_location, 0);

        self.texture_normal.bind_at(1);
        self.program.set_uniform_1i(self.tex_normal_location, 1);

        self.texture_specular.bind_at(2);
        self.program.set_uniform_1i(self.tex_specular_location, 2);

        self.program.set_uniform_matrix_4fv(self.program_view_location, view_matrix);
        self.program.set_uniform_matrix_4fv(self.program_projection_location, proj_matrix);
        self.program.set_uniform_3f(self.camera_pos_location, camera_pos);
        self.vao.bind();

        unsafe {
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0, // staring index
                self.index_count, // number to be rendered
            );
        }
    }
}