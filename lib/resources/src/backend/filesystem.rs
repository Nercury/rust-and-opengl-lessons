use crate::backend::{Backend, BackendSyncPoint, Modification};
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::collections::VecDeque;
use crate::{Error, ResourcePath};
use std::sync::Mutex;

#[cfg(feature = "backend_filesystem_watch")]
mod watch_impl {
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{channel, Receiver, TryRecvError};
    use std::time::{Duration, Instant};
    use notify::{RecommendedWatcher, Watcher as NotifyWatcher, RecursiveMode, DebouncedEvent};
    use crate::backend::{BackendSyncPoint, Modification};
    use crate::{ResourcePathBuf};

    pub struct Watcher {
        root_path: PathBuf,
        _watcher: RecommendedWatcher,
        receiver: Receiver<DebouncedEvent>,
        outdated_at: Option<Instant>,
    }

    impl Watcher {
        pub fn new(root_path: &Path) -> Option<Watcher> {
            let (tx, rx) = channel();

            let mut watcher: RecommendedWatcher = NotifyWatcher::new(tx, Duration::from_millis(50))
                .map_err(|e| error!("failed to create watcher for {:?}, {:?}", root_path, e))
                .ok()?;
            watcher.watch(root_path, RecursiveMode::Recursive).ok()?;

            Some(Watcher {
                root_path: root_path.into(),
                _watcher: watcher,
                receiver: rx,
                outdated_at: None,
            })
        }

        pub fn notify_changes_synced(&mut self, point: BackendSyncPoint) {
            if let Some(last_outdated) = self.outdated_at {
                if point.instant == last_outdated {
                    self.outdated_at = None;
                }
            }
        }

        pub fn new_changes(&mut self, queue: &mut VecDeque<Modification>) -> Option<BackendSyncPoint> {
            let mut something_outdated = false;

            loop {
                match self.receiver.try_recv() {
                    Ok(event) => {
                        match event {
                            DebouncedEvent::Create(path) => {
                                if let Some(resource_path) = ResourcePathBuf::from_filesystem_path(&self.root_path, &path) {
                                    queue.push_back(Modification::Create(resource_path));
                                    something_outdated = true;
                                } else {
                                    warn!("unrecognised resource path {:?} for {} event", path, "Create")
                                }
                            },
                            DebouncedEvent::Write(path) => {
                                if let Some(resource_path) = ResourcePathBuf::from_filesystem_path(&self.root_path, &path) {
                                    queue.push_back(Modification::Write(resource_path));
                                    something_outdated = true;
                                } else {
                                    warn!("unrecognised resource path {:?} for {} event", path, "Write")
                                }
                            },
                            DebouncedEvent::Remove(path) => {
                                if let Some(resource_path) = ResourcePathBuf::from_filesystem_path(&self.root_path, &path) {
                                    queue.push_back(Modification::Remove(resource_path));
                                    something_outdated = true;
                                } else {
                                    warn!("unrecognised resource path {:?} for {} event", path, "Remove")
                                }
                            },
                            DebouncedEvent::Rename(from_path, to_path) => {
                                match (ResourcePathBuf::from_filesystem_path(&self.root_path, &from_path), ResourcePathBuf::from_filesystem_path(&self.root_path, &to_path)) {
                                    (Some(from), Some(to)) => {
                                        queue.push_back(Modification::Rename { from, to });
                                        something_outdated = true;
                                    },
                                    (None, Some(_)) => warn!("unrecognised resource path {:?} for {} event", from_path, "Rename"),
                                    (Some(_), None) => warn!("unrecognised resource path {:?} for {} event", to_path, "Rename"),
                                    (None, None) => warn!("unrecognised resource paths {:?} and {:?} for Rename event", from_path, to_path),
                                }
                            },
                            _ => (),
                        }
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        error!("filesystem watcher disconnected");
                        break;
                    },
                }
            }

            if something_outdated {
                let outdated_at = Instant::now();

                self.outdated_at = Some(outdated_at);

                Some(BackendSyncPoint { instant: outdated_at })
            } else {
                None
            }
        }
    }
}

#[cfg(not(feature = "backend_filesystem_watch"))]
mod watch_impl {
    use std::collections::VecDeque;
    use crate::backend::{BackendSyncPoint, Modification};

    pub struct Watcher {}

    impl Watcher {
        pub fn notify_changes_synced(&mut self, _point: BackendSyncPoint) {}

        pub fn new_changes(&mut self, _queue: &mut VecDeque<Modification>) -> Option<BackendSyncPoint> {
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

    fn notify_changes_synced(&mut self, point: BackendSyncPoint) {
        if let Some(ref mut watch) = self.watch {
            watch.lock().unwrap().notify_changes_synced(point);
        }
    }

    fn new_changes(&mut self, queue: &mut VecDeque<Modification>) -> Option<BackendSyncPoint> {
        if let Some(ref mut watch) = self.watch {
            watch.lock().unwrap().new_changes(queue)
        } else {
            None
        }
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
