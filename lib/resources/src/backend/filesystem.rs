use std::path::{Path, PathBuf};
use std::{io, fs};
use backend::{Backend, BackendSyncPoint};
use {ResourcePath, Error};
use std::io::Read;

pub struct FileSystem {
    root_path: PathBuf,
    can_write: bool,
    watch: bool,
}

impl FileSystem {
    pub fn from_rel_path<P: AsRef<Path>, RP: AsRef<ResourcePath>>(root_path: P, rel_path: RP) -> FileSystem {
        FileSystem::from_path(resource_name_to_path(root_path.as_ref(), rel_path.as_ref()))
    }

    pub fn from_path<P: AsRef<Path>>(root_path: P) -> FileSystem {
        FileSystem {
            root_path: root_path.as_ref().into(),
            can_write: false,
            watch: false,
        }
    }

    pub fn with_write(mut self) -> Self {
        self.can_write = true;
        self
    }

    pub fn with_watch(mut self) -> Self {
        self.watch = true;
        self
    }
}

impl Backend for FileSystem {
    fn can_write(&self) -> bool {
        self.can_write
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        resource_name_to_path(&self.root_path, path).exists()
    }

    fn notify_changes_synced(&mut self, _point: BackendSyncPoint) {
    }

    fn new_changes(&mut self) -> Option<BackendSyncPoint> {
        None
    }

    fn read_into(&mut self, path: &ResourcePath, mut output: &mut io::Write) -> Result<(), Error> {
        let path = resource_name_to_path(&self.root_path, path);
        let mut reader = io::BufReader::new(fs::File::open(path)?);
        io::copy(&mut reader, &mut output)?;
        Ok(())
    }

    fn write_from(&mut self, _path: &ResourcePath, _buffer: &mut io::Read) -> Result<(), Error> {
        unimplemented!()
    }
}

fn resource_name_to_path(root_dir: &Path, location: &ResourcePath) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.items() {
        path = path.join(part);
    }

    path
}