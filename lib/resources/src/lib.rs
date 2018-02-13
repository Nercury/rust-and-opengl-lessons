use std::path::{Path, PathBuf};
use std::ffi::CString;
use std::fs;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum Error {
    FailedToGetExePath,
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_rel_path<S: AsRef<Path>>(rel_path: S) -> Result<Resources, Error> {
        let exe_file_name = std::env::current_exe()
            .map_err(|_| Error::FailedToGetExePath)?;

        let exe_path = exe_file_name.parent()
            .map(|path| path.join(rel_path))
            .ok_or(Error::FailedToGetExePath)?;

        Ok(Resources {
            root_path: exe_path
        })
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<CString, std::io::Error> {
        let mut file = fs::File::open(
            resource_name_to_path(
                &self.root_path,
                resource_name
            )
        )?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_end(&mut buffer)?;

        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData, "can not load CString from file that contains nil")
            );
        }

        Ok(unsafe { CString::from_vec_unchecked(buffer) })
    }
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}