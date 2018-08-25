use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::io;
use backend::{Backend, BackendSyncPoint};
use {ResourcePath, Error};

struct Shared {
    root_path: PathBuf,
    can_write: bool,
    watch: bool,
}

impl Shared {
    pub fn new(root_path: PathBuf) -> Shared {
        Shared {
            root_path,
            can_write: false,
            watch: false,
        }
    }

    pub fn set_write(&mut self, flag: bool) {
        self.can_write = flag;
    }

    pub fn set_watch(&mut self, flag: bool) {
        self.watch = flag;
    }

    pub fn resource_exists(&self, path: &ResourcePath) -> bool {
        resource_name_to_path(&self.root_path, path).exists()
    }
}

pub struct FileSystem {
    shared: Arc<RwLock<Shared>>,
}

impl FileSystem {
    pub fn from_rel_path<P: AsRef<Path>, RP: AsRef<ResourcePath>>(root_path: P, rel_path: RP) -> FileSystem {
        FileSystem::from_path(resource_name_to_path(root_path.as_ref(), rel_path.as_ref()))
    }

    pub fn from_path<P: AsRef<Path>>(root_path: P) -> FileSystem {
        FileSystem {
            shared: Arc::new(RwLock::new(Shared::new(root_path.as_ref().into()))),
        }
    }

    pub fn with_write(self) -> Self {
        self.shared.write().expect("failed to lock FileSystem for write")
            .set_write(true);
        self
    }

    pub fn with_watch(self) -> Self {
        self.shared.write().expect("failed to lock FileSystem for write")
            .set_watch(true);
        self
    }
}

impl Backend for FileSystem {
    fn can_write(&self) -> bool {
        self.shared.read().expect("failed to lock FileSystem for read")
            .can_write
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.shared.read().expect("failed to lock FileSystem for read")
            .resource_exists(path)
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