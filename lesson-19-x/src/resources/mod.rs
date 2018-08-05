use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use std::ffi;
use image;

mod error;
pub mod obj;

pub use self::error::Error;

pub struct ImportedModels {
    pub imported_from_resource_path: String,
    pub models: Vec<::tobj::Model>,
    pub materials: Vec<::tobj::Material>,
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_relative_exe_path(rel_path: &str) -> Result<Resources, Error> {
        let exe_file_name = ::std::env::current_exe()
            .map_err(|_| Error::FailedToGetExePath)?;

        let exe_path = exe_file_name.parent()
            .ok_or(Error::FailedToGetExePath)?;

        Ok(Resources {
            root_path: resource_name_to_path(&exe_path, rel_path)
        })
    }

    pub fn from_exe_path() -> Result<Resources, Error> {
        Resources::from_relative_exe_path("")
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let mut file = fs::File::open(
            resource_name_to_path(&self.root_path,resource_name)
        )?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity(
            file.metadata()?.len() as usize + 1
        );
        file.read_to_end(&mut buffer)?;

        // check for nul byte
        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(Error::FileContainsNil);
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }

    pub fn load_rgb_image(&self, image_file_name: &str) -> Result<image::RgbImage, Error> {
        let img = image::open(
            resource_name_to_path(&self.root_path,image_file_name)
        )?;

        Ok(img.to_rgb())
    }

    pub fn load_rgba_image(&self, image_file_name: &str) -> Result<image::RgbaImage, Error> {
        let img = image::open(
            resource_name_to_path(&self.root_path,image_file_name)
        )?;

        if let image::ColorType::RGBA(_) = img.color() {
            Ok(img.to_rgba())
        } else {
            Err(Error::ImageIsNotRgba { name: image_file_name.into() })
        }
    }

    pub fn load_obj(&self, obj_file_name: &str) -> Result<ImportedModels, Error> {
        obj::ModelsWithMaterials::load(&resource_name_to_path(&self.root_path, obj_file_name))
            .map(|m| ImportedModels {
                models: m.models,
                materials: m.materials,
                imported_from_resource_path: resource_path_parent(obj_file_name),
            })
            .map_err(|e| Error::FailedToLoadObj { name: obj_file_name.into(), inner: e })
    }
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}

fn resource_path_parent(path: &str) -> String {
    match path.rfind('/') {
        None => "".into(),
        Some(index) => path[..index].into(),
    }
}