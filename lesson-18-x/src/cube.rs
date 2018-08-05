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
    #[location = "2"]
    normal: data::f32_f32_f32,
    #[location = "3"]
    uv: data::f16_f16,
}

pub struct Cube {
    program: render_gl::Program,
    texture: Option<render_gl::Texture>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    camera_pos_location: Option<i32>,
    tex_face_location: Option<i32>,
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

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let camera_pos_location = program.get_uniform_location("CameraPos");
        let tex_face_location = program.get_uniform_location("TexFace");

        let imported_models = res.load_obj("objs/dice.obj")?;

        // take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); // it is first or None

        let texture = match material {
            Some(material) => if &material.diffuse_texture == "" {
                None
            } else {
                Some(render_gl::Texture::from_res_rgb(
                    &[&imported_models.imported_from_resource_path[..], "/", &material.diffuse_texture[..]].concat()
                ).load(gl, res)?)
            },
            None => None,
        };

        // match mesh to material id and get the mesh
        let mesh = imported_models.models.into_iter()
            .filter(|model| model.mesh.material_id == material_index)
            .next()
            .expect("expected obj file to contain a mesh").mesh;

        let vbo_data = mesh.positions.chunks(3)
            .zip(mesh.normals.chunks(3))
            .zip(mesh.texcoords.chunks(2))
            .map(|((p, n), t)|
                Vertex { pos: (p[0], p[1], p[2]).into(), normal: (n[0], n[1], n[2]).into(), uv:  (t[0], -t[1]).into() }
            )
            .collect::<Vec<_>>();

        let ebo_data = mesh.indices;

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
            program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            tex_face_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
            _debug_rays: vbo_data.iter().map(|v| debug_lines.ray_marker(
                na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                na::Vector3::new(v.normal.d0, v.normal.d1, v.normal.d2),
                na::Vector4::new(0.5, 0.5, 1.0, 0.6)
            )).collect()
        })
    }

    pub fn render(&self, gl: &gl::Gl, view_matrix: &na::Matrix4<f32>, proj_matrix: &na::Matrix4<f32>, camera_pos: &na::Vector3<f32>) {
        self.program.set_used();

        if let (Some(loc), &Some(ref texture)) = (self.tex_face_location, &self.texture) {
            texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
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
                gl::UNSIGNED_INT, // index type
                ::std::ptr::null() // pointer to indices (we are using ebo configured at vao creation)
            );
        }
    }
}