use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use std::ffi;
use image;
use mesh;

mod error;
mod path;
pub mod obj;

pub use self::error::Error;
pub use self::path::{ResourcePath, ResourcePathBuf};

pub struct ImportedModels {
    pub imported_from_resource_path: ResourcePathBuf,
    pub models: Vec<::tobj::Model>,
    pub materials: Vec<::tobj::Material>,
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_relative_exe_path<P: AsRef<ResourcePath>>(rel_path: P) -> Result<Resources, Error> {
        let exe_file_name = ::std::env::current_exe()
            .map_err(|_| Error::FailedToGetExePath)?;

        let exe_path = exe_file_name.parent()
            .ok_or(Error::FailedToGetExePath)?;

        Ok(Resources {
            root_path: resource_name_to_path(&exe_path, rel_path.as_ref())
        })
    }

    pub fn from_exe_path() -> Result<Resources, Error> {
        Resources::from_relative_exe_path("")
    }

    pub fn load_cstring<P: AsRef<ResourcePath>>(&self, rel_path: P) -> Result<ffi::CString, Error> {
        let mut file = fs::File::open(
            resource_name_to_path(&self.root_path, rel_path.as_ref())
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

    pub fn load_rgb_image<P: AsRef<ResourcePath>>(&self, rel_path: P) -> Result<image::RgbImage, Error> {
        let img = image::open(
            resource_name_to_path(&self.root_path, rel_path.as_ref())
        ).map_err(|e| Error::FailedToLoadImage { name: rel_path.as_ref().to_string(), inner: e })?;

        Ok(img.to_rgb())
    }

    pub fn load_rgba_image<P: AsRef<ResourcePath>>(&self, rel_path: P) -> Result<image::RgbaImage, Error> {
        let img = image::open(
            resource_name_to_path(&self.root_path, rel_path.as_ref())
        ).map_err(|e| Error::FailedToLoadImage { name: rel_path.as_ref().to_string(), inner: e })?;

        if let image::ColorType::RGBA(_) = img.color() {
            Ok(img.to_rgba())
        } else {
            Err(Error::ImageIsNotRgba { name: rel_path.as_ref().to_string() })
        }
    }

    pub fn load_obj<P: AsRef<ResourcePath>>(&self, rel_path: P) -> Result<mesh::MeshSet, Error> {
        obj::ModelsWithMaterials::load(&self.root_path, rel_path.as_ref())
            .map_err(|e| Error::FailedToLoadObj { name: rel_path.as_ref().to_string(), inner: e })
    }
}

fn resource_name_to_path(root_dir: &Path, location: &ResourcePath) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.items() {
        path = path.join(part);
    }

    path
}