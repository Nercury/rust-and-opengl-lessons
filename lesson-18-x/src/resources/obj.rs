use tobj;
use std::path::{PathBuf, Path};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Obj or Mtl load error")]
    LoadError(#[cause] tobj::LoadError),
    #[fail(display = "Obj path {:?} must be absolute", path)]
    ObjPathMustBeAbsolute { path: PathBuf },
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
    pub fn load(path: &Path) -> Result<ModelsWithMaterials, Error> {
        if !path.is_absolute() {
            return Err(Error::ObjPathMustBeAbsolute { path: path.into() });
        }

        let (models, materials) = tobj::load_obj(path)?;

        println!("models: {:?}", models.len());
        for model in &models {
            println!("name: {:?}", model.name);
            println!("positions {:?}", model.mesh.positions);
            println!("normals {:?}", model.mesh.normals);
            println!("texcoords {:?}", model.mesh.texcoords);
            println!("indices {:?}", model.mesh.indices);
            println!("material_id {:?}", model.mesh.material_id);
        }

        println!("materials: {:?}", materials.len());
        for obj in &materials {
            println!("material: {:#?}", obj);
        }

        Ok(ModelsWithMaterials {
            models,
            materials,
        })
    }
}