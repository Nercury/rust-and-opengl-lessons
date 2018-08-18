use gl;
use failure;
use render_gl::{self, DebugLines};
use selection::{self, Selectables, SelectableAABB};
use resources::Resources;
use nalgebra as na;

mod buffers;
mod dice_material;

use self::buffers::{Buffers};

pub struct Dice {
    transform: na::Isometry3<f32>,
    program: render_gl::Program,
    texture: Option<render_gl::Texture>,
    texture_normals: Option<render_gl::Texture>,
    material: dice_material::Material,
    buffers: Buffers,
    debug_tangent_normals: render_gl::RayMarkers,
    selectable_aabb: Option<SelectableAABB>,
}

impl Dice {
    pub fn new(res: &Resources, gl: &gl::Gl, debug_lines: &DebugLines, selectables: &Selectables) -> Result<Dice, failure::Error> {

        // set up shader program

        let program = render_gl::Program::from_res(gl, res, "shaders/shiny")?;
        let p_material = dice_material::Material::load_for(&program);

        // this loader does not support file names with spaces
        let imported_models = res.load_obj("objs/dice.obj")?;

        // take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); // it is first or None

        let texture = material.as_ref()
            .and_then(|m| m.diffuse_map.as_ref()
                .and_then(|resource_path|
                    render_gl::Texture::from_res_rgb(&resource_path)
                        .with_gen_mipmaps()
                        .load(gl, res)
                        .map_err(|e| println!("Error loading {}: {}", resource_path, e))
                        .ok()
                ));
        let texture_normals = material.as_ref()
            .and_then(|m| m.bump_map.as_ref()
                .and_then(|resource_path|
                    render_gl::Texture::from_res_rgb(&resource_path)
                        .with_gen_mipmaps()
                        .load(gl, res)
                        .map_err(|e| println!("Error loading {}: {}", resource_path, e))
                        .ok()
                ));

        // match mesh to material id and get the mesh
        let mesh = imported_models.meshes.into_iter()
            .filter(|model| model.material_index == material_index)
            .next()
            .expect("expected obj file to contain a mesh");

        let initial_isometry = na::Isometry3::identity();

        Ok(Dice {
            transform: initial_isometry,
            texture,
            texture_normals,
            program,
            material: p_material,
            buffers: Buffers::new(gl, &mesh),
            debug_tangent_normals: debug_lines.ray_markers(
                initial_isometry,
                mesh.vertices.iter().filter_map(|v| v.normal.map(|n| (v.pos, n))).map(|(p, n)| (
                    p,
                    n * 0.2,
                    na::Vector4::new(0.0, 0.0, 1.0, 1.0),
                )).chain(mesh.vertices.iter().filter_map(|v| v.tangents.map(|t| (v.pos, t.tangent))).map(|(p, n)| (
                    p,
                    n * 0.2,
                    na::Vector4::new(0.0, 1.0, 0.0, 1.0),
                )))
            ),
            selectable_aabb: mesh.aabb().map(|aabb| selectables.selectable(aabb, initial_isometry)),
        })
    }

    pub fn update(&mut self, _delta: f32) {
        loop {
            let action = self.selectable_aabb.as_ref().and_then(|s| s.drain_pending_action());

            match action {
                Some(selection::Action::Click) => { self.selectable_aabb.as_ref().map(|s| s.select()); },
                Some(selection::Action::Drag { new_isometry }) => self.set_transform(new_isometry),
                _ => break,
            }
        }
    }

    pub fn set_transform(&mut self, isometry: na::Isometry3<f32>) {
        self.transform = isometry;
        if let Some(ref selectable) = self.selectable_aabb {
            selectable.update_isometry(isometry);
        }
        self.debug_tangent_normals.update_isometry(isometry);
    }

    pub fn render(&self, gl: &gl::Gl, viewprojection_matrix: &na::Matrix4<f32>, camera_pos: &na::Vector3<f32>) {
        self.program.set_used();

        self.material.bind(
            &self.program,
            viewprojection_matrix, &self.transform.to_homogeneous(), camera_pos,
            &self.texture, &self.texture_normals
        );

        self.buffers.render(gl);
    }
}