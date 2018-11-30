use failure;
use gl;
use crate::mesh;
use nalgebra as na;
use crate::render_gl::{self, buffer, data};
use crate::resources::Resources;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    uv: data::f16_f16,
    #[location = "2"]
    t: data::f32_f32_f32,
    #[location = "3"]
    b: data::f32_f32_f32,
    #[location = "4"]
    n: data::f32_f32_f32,
}

pub struct Cube {
    program: render_gl::Program,
    texture: Option<render_gl::Texture>,
    texture_normals: Option<render_gl::Texture>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    camera_pos_location: Option<i32>,
    texture_location: Option<i32>,
    texture_normals_location: Option<i32>,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
    _debug_tangent_normals: Vec<render_gl::RayMarker>,
    _debug_normals: Vec<render_gl::RayMarker>,
}

impl Cube {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        debug_lines: &render_gl::DebugLines,
    ) -> Result<Cube, failure::Error> {
        // set up shader program

        let program = render_gl::Program::from_res(gl, res, "shaders/cube")?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let camera_pos_location = program.get_uniform_location("CameraPos");
        let texture_location = program.get_uniform_location("Texture");
        let texture_normals_location = program.get_uniform_location("Normals");

        // this loader does not support file names with spaces
        let imported_models = res.load_obj("objs/dice.obj")?;

        // take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); // it is first or None

        let texture = material.as_ref().and_then(|m| {
            m.diffuse_map.as_ref().and_then(|resource_path| {
                render_gl::Texture::from_res_rgb(&resource_path)
                    .with_gen_mipmaps()
                    .load(gl, res)
                    .map_err(|e| println!("Error loading {}: {}", resource_path, e))
                    .ok()
            })
        });
        let texture_normals = material.as_ref().and_then(|m| {
            m.bump_map.as_ref().and_then(|resource_path| {
                render_gl::Texture::from_res_rgb(&resource_path)
                    .with_gen_mipmaps()
                    .load(gl, res)
                    .map_err(|e| println!("Error loading {}: {}", resource_path, e))
                    .ok()
            })
        });

        // match mesh to material id and get the mesh
        let mesh = imported_models
            .meshes
            .into_iter()
            .filter(|model| model.material_index == material_index)
            .next()
            .expect("expected obj file to contain a mesh");

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
                Vertex {
                    pos: (v.pos.x, v.pos.y, v.pos.z).into(),
                    uv: (uv.x, -uv.y).into(),
                    t: (tv.tangent.x, tv.tangent.y, tv.tangent.z).into(),
                    b: (tv.bitangent.x, tv.bitangent.y, tv.bitangent.z).into(),
                    n: (normal.x, normal.y, normal.z).into(),
                }
            }).collect::<Vec<_>>();

        let ebo_data = mesh.triangle_indices();

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
            texture_normals,
            program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            texture_location,
            texture_normals_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
            _debug_tangent_normals: vbo_data
                .iter()
                .map(|v| {
                    debug_lines.ray_marker(
                        na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                        na::Vector3::new(v.n.d0, v.n.d1, v.n.d2) * 0.2,
                        na::Vector4::new(0.0, 0.0, 1.0, 1.0),
                    )
                }).chain(vbo_data.iter().map(|v| {
                    debug_lines.ray_marker(
                        na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                        na::Vector3::new(v.t.d0, v.t.d1, v.t.d2) * 0.2,
                        na::Vector4::new(0.0, 1.0, 0.0, 1.0),
                    )
                })).chain(vbo_data.iter().map(|v| {
                    debug_lines.ray_marker(
                        na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                        na::Vector3::new(v.b.d0, v.b.d1, v.b.d2) * 0.2,
                        na::Vector4::new(1.0, 0.0, 0.0, 1.0),
                    )
                })).collect(),
            _debug_normals: vec![],
            //            _debug_normals: vertices.iter().map(|v| debug_lines.ray_marker(
            //                na::Point3::new(v.pos.x, v.pos.y, v.pos.z),
            //                na::Vector3::new(v.normal.x, v.normal.y, v.normal.z) * 0.2,
            //                na::Vector4::new(0.1, 0.9, 0.1, 0.6)
            //            )).collect()
        })
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        camera_pos: &na::Vector3<f32>,
    ) {
        self.program.set_used();

        if let (Some(loc), &Some(ref texture)) = (self.texture_location, &self.texture) {
            texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let (Some(loc), &Some(ref texture)) =
            (self.texture_normals_location, &self.texture_normals)
        {
            texture.bind_at(1);
            self.program.set_uniform_1i(loc, 1);
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
