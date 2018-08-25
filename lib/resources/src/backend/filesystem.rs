use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::io;
use backend::{Backend, BackendSyncPoint};
use {ResourcePath, Error};

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

    fn notify_changes_synced(&mut self, point: BackendSyncPoint) {
        unimplemented!()
    }

    fn new_changes(&mut self) -> Option<BackendSyncPoint> {
        unimplemented!()
    }

    fn read_into(&mut self, path: &ResourcePath, output: &mut io::Write) -> Result<(), Error> {
        unimplemented!()
    }

    fn write_from(&mut self, path: &ResourcePath, buffer: &mut io::Read) -> Result<(), Error> {
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