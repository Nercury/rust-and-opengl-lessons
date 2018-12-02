use crate::backend::{Backend, BackendSyncPoint};
use std::path::{Path, PathBuf};
use std::{fs, io};
use crate::{Error, ResourcePath};
use std::sync::Mutex;

#[cfg(feature = "backend_filesystem_watch")]
mod watch_impl {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{channel, Receiver};
    use std::time::Duration;
    use notify::{RecommendedWatcher, Watcher as NotifyWatcher, RecursiveMode, DebouncedEvent};

    pub struct Watcher {
        root_path: PathBuf,
        watcher: RecommendedWatcher,
        receiver: Receiver<DebouncedEvent>,
    }

    impl Watcher {
        pub fn new(root_path: &Path) -> Option<Watcher> {
            let (tx, rx) = channel();

            let mut watcher: RecommendedWatcher = NotifyWatcher::new(tx, Duration::from_secs(2))
                .map_err(|e| error!("faled to create watcher for {:?}", root_path))
                .ok()?;
            watcher.watch(root_path, RecursiveMode::Recursive).ok()?;

            Some(Watcher {
                root_path: root_path.into(),
                watcher,
                receiver: rx,
            })
        }
    }
}

#[cfg(not(feature = "backend_filesystem_watch"))]
mod watch_impl {
    use std::path::{Path};

    pub struct Watcher {}

    impl Watcher {
        pub fn new(_root_path: &Path) -> Option<Watcher> {
            None
        }
    }
}

pub struct FileSystem {
    root_path: PathBuf,
    can_write: bool,
    watch: Option<Mutex<watch_impl::Watcher>>,
}

impl FileSystem {
    pub fn from_rel_path<P: AsRef<Path>, RP: AsRef<ResourcePath>>(
        root_path: P,
        rel_path: RP,
    ) -> FileSystem {
        FileSystem::from_path(resource_name_to_path(root_path.as_ref(), rel_path.as_ref()))
    }

    pub fn from_path<P: AsRef<Path>>(root_path: P) -> FileSystem {
        FileSystem {
            root_path: root_path.as_ref().into(),
            can_write: false,
            watch: None,
        }
    }

    pub fn with_write(mut self) -> Self {
        self.can_write = true;
        self
    }

    #[cfg(feature = "backend_filesystem_watch")]
    pub fn with_watch(mut self) -> Self {
        self.watch = watch_impl::Watcher::new(&self.root_path).map(|v| Mutex::new(v));
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

    fn notify_changes_synced(&mut self, _point: BackendSyncPoint) {}

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
