use tobj;
use mesh;
use std::path::{Path};
use resources::{ResourcePath, ResourcePathBuf};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Obj or Mtl load error")]
    LoadError(#[cause] tobj::LoadError),
    #[fail(display = "Resource path must not be empty")]
    ResourcePathMustNotBeEmpty,
}

impl From<tobj::LoadError> for Error {
    fn from(other: tobj::LoadError) -> Self {
        Error::LoadError(other)
    }
}

pub struct ModelsWithMaterials {
    pub models: Vec<tobj::Model>,
    pub materials: Vec<tobj::Material>,
}

impl ModelsWithMaterials {
    pub fn load(root_path: &Path, resource_path: &ResourcePath) -> Result<mesh::MeshSet, Error> {
        let fs_path = super::resource_name_to_path(root_path, resource_path);
        let resource_dir = resource_path.parent().ok_or(Error::ResourcePathMustNotBeEmpty)?;

        let (models, materials) = tobj::load_obj(&fs_path)?;

        let mapped_materials = materials
            .into_iter()
            .map(|m| mesh::Material {
                name: Some(m.name),
                diffuse_map: if &m.diffuse_texture == "" {
                    None
                } else {
                    Some(resource_dir.join(&platform_path_to_rel_resource_path(&m.diffuse_texture)))
                },
                bump_map: match m.unknown_param.iter()
                    .filter(|(k, _)|
                        k.to_lowercase() == "map_bump" || k.to_lowercase() == "bump"
                    ).map(|(_, v)| v)
                    .next()
                    {
                        Some(ref name) => Some(resource_dir.join(&platform_path_to_rel_resource_path(name))),
                        None => None,
                    }
            })
            .collect::<Vec<_>>();

        let mapped_meshes = models
            .into_iter()
            .map(|m| map_model_to_mesh(m, &mapped_materials))
            .collect::<Vec<_>>();

        Ok(mesh::MeshSet {
            materials: mapped_materials,
            meshes: mapped_meshes,
        })
    }
}

fn map_model_to_mesh(model: tobj::Model, mapped_materials: &[mesh::Material]) -> mesh::Mesh {

    let normals = if model.mesh.normals.len() == 0 { None } else { Some(model.mesh.normals) };
    let texcoords = if model.mesh.texcoords.len() == 0 { None } else { Some(model.mesh.texcoords) };
    let mut vertices = Vec::with_capacity(model.mesh.positions.len() / 3);

    for (index, p) in model.mesh.positions.chunks(3).enumerate() {
        vertices.push(mesh::Vertex {
            pos: [p[0], p[1], p[2]].into(),
            normal: normals.as_ref().map(|n| {
                let index = index * 3;
                [n[index + 0], n[index + 1], n[index + 2]].into()
            }),
            tangents: None,
            uv: texcoords.as_ref().map(|t| {
                let index = index * 2;
                [t[index + 0], t[index + 1]].into()
            }),
        });
    }

    let primitives = model.mesh.indices.chunks(3)
        .filter_map(|c| if c.len() == 3 {
            Some(mesh::Primitive::Triangle(c[0], c[1], c[2]))
        } else {
            None
        })
        .collect::<Vec<_>>();

    let mut mesh = mesh::Mesh {
        name: Some(model.name),
        vertices,
        primitives,
        material_index: match model.mesh.material_id {
            Some(id) => if id >= mapped_materials.len() { None } else { Some(id) },
            None => None,
        }
    };

    mesh.calculate_tangents();

    mesh
}

fn platform_path_to_rel_resource_path(value: &str) -> ResourcePathBuf {
    value.replace('\\', "/").into()
}