use nalgebra as na;
use crate::render_gl;

pub struct Material {
    texture_location: Option<i32>,
    texture_normals_location: Option<i32>,

    program_viewprojection_location: Option<i32>,
    program_model_location: Option<i32>,
    camera_pos_location: Option<i32>,
}

impl Material {
    pub fn load_for(program: &render_gl::Program) -> Material {
        Material {
            texture_location: program.get_uniform_location("Texture"),
            texture_normals_location: program.get_uniform_location("Normals"),

            program_viewprojection_location: program.get_uniform_location("ViewProjection"),
            program_model_location: program.get_uniform_location("Model"),
            camera_pos_location: program.get_uniform_location("CameraPos"),
        }
    }

    pub fn bind(
        &self,
        program: &render_gl::Program,
        viewprojection_matrix: &na::Matrix4<f32>,
        model_matrix: &na::Matrix4<f32>,
        camera_pos: &na::Vector3<f32>,
        texture: &Option<render_gl::Texture>,
        texture_normals: &Option<render_gl::Texture>,
    ) {
        if let (Some(loc), &Some(ref texture)) = (self.texture_location, texture) {
            texture.bind_at(0);
            program.set_uniform_1i(loc, 0);
        }

        if let (Some(loc), &Some(ref texture)) = (self.texture_normals_location, texture_normals) {
            texture.bind_at(1);
            program.set_uniform_1i(loc, 1);
        }

        if let Some(loc) = self.program_viewprojection_location {
            program.set_uniform_matrix_4fv(loc, viewprojection_matrix);
        }
        if let Some(loc) = self.program_model_location {
            program.set_uniform_matrix_4fv(loc, model_matrix);
        }
        if let Some(loc) = self.camera_pos_location {
            program.set_uniform_3f(loc, camera_pos);
        }
    }
}
