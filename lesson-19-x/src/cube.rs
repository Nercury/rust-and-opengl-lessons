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
    uv: data::f16_f16,
    #[location = "2"]
    t: data::f32_f32_f32,
    #[location = "3"]
    b: data::f32_f32_f32,
    #[location = "4"]
    n: data::f32_f32_f32,
}

#[derive(Copy, Clone, Debug)]
struct ImportedVertex {
    pos: na::Vector3<f32>,
    uv: na::Vector2<f32>,
    normal: na::Unit<na::Vector3<f32>>,
    tangent_vectors: Option<TangentVectors>,
}

#[derive(Copy, Clone, Debug)]
struct TangentVectors {
    t: na::Unit<na::Vector3<f32>>,
    b: na::Unit<na::Vector3<f32>>,
    n: na::Unit<na::Vector3<f32>>,
}

impl Default for TangentVectors {
    fn default() -> Self {
        TangentVectors { t: na::Unit::new_normalize([0.0, 0.0, 0.0].into()), b: na::Unit::new_normalize([0.0, 0.0, 0.0].into()), n: na::Unit::new_normalize([0.0, 0.0, 0.0].into()) }
    }
}

impl TangentVectors {
    pub fn from_triangle(p0: &ImportedVertex, p1: &ImportedVertex, p2: &ImportedVertex) -> TangentVectors {
        // position differences p1->p2 and p1->p3

        let v = p1.pos - p0.pos;
        let w = p2.pos - p0.pos;

        // texture offset p1->p2 and p1->p3
        let mut sx = p1.uv.x - p0.uv.x;
        let mut sy = p1.uv.y - p0.uv.y;
        let mut tx = p2.uv.x - p0.uv.x;
        let mut ty = p2.uv.y - p0.uv.y;

        let dir_correction = if (tx * sy - ty * sx) < 0.0 { -1.0 } else { 1.0 };

        // when t1, t2, t3 in same position in UV space, just use default UV direction.
        if sx * ty == sy * tx {
            sx = 0.0; sy = 1.0;
            tx = 1.0; ty = 0.0;
        }

        let tangent = na::Unit::new_normalize(na::Vector3::new(
            (w.x * sy - v.x * ty) * dir_correction,
            (w.y * sy - v.y * ty) * dir_correction,
            (w.z * sy - v.z * ty) * dir_correction
        ));
        let bitangent = -na::Unit::new_normalize(na::Vector3::new(
                     // ^--- WTF
            (w.x * sx - v.x * tx) * dir_correction,
            (w.y * sx - v.y * tx) * dir_correction,
            (w.z * sx - v.z * tx) * dir_correction
        ));

        let tangent_normal = na::Unit::new_normalize(tangent.cross(&bitangent));

        TangentVectors {
            t: tangent,
            b: bitangent,
            n: tangent_normal
        }
    }
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
    pub fn new(res: &Resources, gl: &gl::Gl, debug_lines: &render_gl::DebugLines) -> Result<Cube, failure::Error> {

        // set up shader program

        let program = render_gl::Program::from_res(gl, res, "shaders/cube")?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let camera_pos_location = program.get_uniform_location("CameraPos");
        let texture_location = program.get_uniform_location("Texture");
        let texture_normals_location = program.get_uniform_location("Normals");

        let imported_models = res.load_obj("objs/dice.obj")?;

        // take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); // it is first or None

        let texture = match material {
            Some(ref material) => if &material.diffuse_texture == "" {
                None
            } else {
                Some(render_gl::Texture::from_res_rgb(
                    &[&imported_models.imported_from_resource_path[..], "/", &material.diffuse_texture[..]].concat()
                ).load(gl, res)?)
            },
            None => None,
        };
        let texture_normals = match material {
            Some(ref material) => match material.unknown_param.get("map_Bump") {
                Some(ref name) => Some(render_gl::Texture::from_res_rgb(
                    &[&imported_models.imported_from_resource_path[..], "/", &name[..]].concat()
                ).load(gl, res)?),
                None => None,
            },
            None => None,
        };
        if texture_normals.is_none() {
            panic!("Normals failed");
        }

        // match mesh to material id and get the mesh
        let mesh = imported_models.models.into_iter()
            .filter(|model| model.mesh.material_id == material_index)
            .next()
            .expect("expected obj file to contain a mesh").mesh;

        let mut vertices = mesh.positions.chunks(3)
            .zip(mesh.normals.chunks(3))
            .zip(mesh.texcoords.chunks(2))
            .map(|((p, n), t)|
                ImportedVertex {
                    pos: [p[0], p[1], p[2]].into(),
                    uv:  [t[0], t[1]].into(),
                    normal: na::Unit::new_normalize([n[0], n[1], n[2]].into()),
                    tangent_vectors: None,
                }
            )
            .collect::<Vec<_>>();

        for chunk in mesh.indices.chunks(3) {
            if let &[ai, bi, ci] = chunk {
                let a = vertices[ai as usize];
                let b = vertices[bi as usize];
                let c = vertices[ci as usize];

                // this was shamelessly "inspired" by assimp library https://github.com/assimp/assimp/blob/master/code/CalcTangentsProcess.cpp

                let face_tangent_vectors = TangentVectors::from_triangle(&a, &b, &c);

                fn is_special_float(v: f32) -> bool {
                    v.is_nan() || v.is_infinite()
                }

                for (v, i) in &[(a, ai), (b, bi), (c, ci)] {
                    // project tangent and bitangent into the plane formed by the vertex' normal
                    let mut local_tangent = face_tangent_vectors.t.unwrap() - v.normal.unwrap() * (face_tangent_vectors.t.dot(&v.normal));
                    let mut local_bitangent = face_tangent_vectors.b.unwrap() - v.normal.unwrap() * (face_tangent_vectors.b.dot(&v.normal));
                    local_tangent.normalize_mut(); local_bitangent.normalize_mut();

                    // reconstruct tangent/bitangent according to normal and bitangent/tangent when it's infinite or NaN.
                    let invalid_tangent = is_special_float(local_tangent.x) || is_special_float(local_tangent.y) || is_special_float(local_tangent.z);
                    let invalid_bitangent = is_special_float(local_bitangent.x) || is_special_float(local_bitangent.y) || is_special_float(local_bitangent.z);

                    if invalid_tangent != invalid_bitangent {
                        if invalid_tangent {
                            local_tangent = v.normal.cross(&local_bitangent);
                            local_tangent.normalize_mut();
                        } else {
                            local_bitangent = local_tangent.cross(&v.normal);
                            local_bitangent.normalize_mut();
                        }
                    }

                    let face_tangent_vectors = TangentVectors {
                        t: na::Unit::new_normalize(local_tangent),
                        b: na::Unit::new_normalize(local_bitangent),
                        n: v.normal,
                    };

                    {
                        let ref mut existing_tangent_vectors = vertices[*i as usize].tangent_vectors;
                        if existing_tangent_vectors.is_some() {
                            println!("Existing vectors {}, new {}", existing_tangent_vectors.unwrap().n.unwrap(), face_tangent_vectors.n.unwrap());
                        } else {
                            *existing_tangent_vectors = Some(face_tangent_vectors);
                        }
                    }
                }
            }
        }

        let vbo_data = vertices.clone()
            .into_iter()
            .map(|v| {
                let tv = v.tangent_vectors.unwrap_or_else(|| {
                    println!("Missing tangent vectors");
                    TangentVectors::default()
                });
                Vertex {
                    pos: (v.pos.x, v.pos.y, v.pos.z).into(),
                    uv: (v.uv.x, -v.uv.y).into(),
                    t: (tv.t.x, tv.t.y, tv.t.z).into(),
                    b: (tv.b.x, tv.b.y, tv.b.z).into(),
                    n: (tv.n.x, tv.n.y, tv.n.z).into()
                }
            })
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
            _debug_tangent_normals: vbo_data.iter().map(|v| debug_lines.ray_marker(
                na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                na::Vector3::new(v.n.d0, v.n.d1, v.n.d2) * 0.2,
                na::Vector4::new(0.0, 0.0, 1.0, 1.0)
            )).chain(vbo_data.iter().map(|v| debug_lines.ray_marker(
                na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                na::Vector3::new(v.t.d0, v.t.d1, v.t.d2) * 0.2,
                na::Vector4::new(0.0, 1.0, 0.0, 1.0)
            ))).chain(vbo_data.iter().map(|v| debug_lines.ray_marker(
                na::Point3::new(v.pos.d0, v.pos.d1, v.pos.d2),
                na::Vector3::new(v.b.d0, v.b.d1, v.b.d2) * 0.2,
                na::Vector4::new(1.0, 0.0, 0.0, 1.0)
            )))
                .collect(),
            _debug_normals: vec![],
//            _debug_normals: vertices.iter().map(|v| debug_lines.ray_marker(
//                na::Point3::new(v.pos.x, v.pos.y, v.pos.z),
//                na::Vector3::new(v.normal.x, v.normal.y, v.normal.z) * 0.2,
//                na::Vector4::new(0.1, 0.9, 0.1, 0.6)
//            )).collect()
        })
    }

    pub fn render(&self, gl: &gl::Gl, view_matrix: &na::Matrix4<f32>, proj_matrix: &na::Matrix4<f32>, camera_pos: &na::Vector3<f32>) {
        self.program.set_used();

        if let (Some(loc), &Some(ref texture)) = (self.texture_location, &self.texture) {
            texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let (Some(loc), &Some(ref texture)) = (self.texture_normals_location, &self.texture_normals) {
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