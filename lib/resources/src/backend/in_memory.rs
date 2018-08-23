use backend::{Writer, Reader, Backend, NotifyDidRead, NotifyDidWrite, BackendSyncPoint};
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::io;
use std::time::Instant;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash;
use {ResourcePathBuf, ResourcePath, Error};

#[derive(Debug)]
struct Shared {
    map: HashMap<ResourcePathBuf, Vec<u8>, BuildHasherDefault<XxHash>>,
    unsynced_change_time: Option<BackendSyncPoint>,
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            map: HashMap::default(),
            unsynced_change_time: None,
        }
    }

    pub fn insert(&mut self, key: &ResourcePath, value: &[u8]) {
        self.map.insert(key.as_ref().into(), value.into());
    }
}

#[derive(Debug)]
pub struct InMemory {
    shared: Arc<RwLock<Shared>>,
}

impl InMemory {
    pub fn new() -> InMemory {
        InMemory {
            shared: Arc::new(RwLock::new(Shared::new()))
        }
    }

    pub fn with<P: AsRef<ResourcePath>>(self, key: P, value: &[u8]) -> Self {
        self.shared.write()
            .expect("failed to lock InMemory for write")
            .insert(key.as_ref(), value);
        self
    }
}

impl Backend for InMemory {
    fn can_write(&self) -> bool {
        true
    }

    fn reader(&self, path: &ResourcePath, modification_time: Option<Instant>, completion_listener: Box<NotifyDidRead>) -> Option<Box<Reader>> {
        Some(Box::new(InMemoryReader {
            shared: self.shared.clone(),
            path: path.into(),
            completion_listener,
            did_read: false,
            modification_time,
        }) as Box<Reader>)
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.shared.read().expect("failed to lock InMemory for read")
            .map.contains_key::<ResourcePath>( path.as_clean_str().as_ref())
    }

    fn writer(&self, path: &ResourcePath, completion_listener: Box<NotifyDidWrite>) -> Option<Box<Writer>> {
        Some(Box::new(InMemoryWriter {
            shared: self.shared.clone(),
            path: Some(path.into()),
            completion_listener,
            did_write: false,
        }) as Box<Writer>)
    }

    fn notify_changes_synced(&self, point: BackendSyncPoint) {
        let mut shared_ref = self.shared.write().expect("failed to lock InMemory for write");

        if shared_ref.unsynced_change_time == Some(point) {
            shared_ref.unsynced_change_time = None;
        }
    }

    fn new_changes(&self) -> Option<BackendSyncPoint> {
        self.shared.read().expect("failed to lock InMemory for read")
            .unsynced_change_time
    }
}

struct InMemoryReader {
    shared: Arc<RwLock<Shared>>,
    path: ResourcePathBuf,
    completion_listener: Box<NotifyDidRead>,
    did_read: bool,
    modification_time: Option<Instant>,
}

impl Reader for InMemoryReader {
    fn read_into(mut self: Box<Self>, output: &mut io::Write) -> Result<(), Error> {
        {
            let shared = self.shared.read().expect("failed to lock InMemory for read");
            let item_ref = match shared.map.get(&self.path) {
                None => return Err(Error::ItemAtPathHasGoneAway { path: self.path.clone() }),
                Some(val) => val
            };
            output.write_all(&item_ref)?;
        }
        self.did_read = true;
        Ok(())
    }
}

impl Drop for InMemoryReader {
    fn drop(&mut self) {
        if self.did_read {
            self.completion_listener.notify_did_read(self.modification_time);
        }
    }
}

struct InMemoryWriter {
    shared: Arc<RwLock<Shared>>,
    path: Option<ResourcePathBuf>,
    completion_listener: Box<NotifyDidWrite>,
    did_write: bool,
}

impl Writer for InMemoryWriter {
    fn write_from(mut self: Box<Self>, buffer: &mut io::Read) -> Result<(), Error> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        {
            let path = ::std::mem::replace(&mut self.path, None).expect("writer should be created with path");
            let mut shared = self.shared.write().expect("failed to lock InMemory for write");
            shared.map.insert(path, data);
            shared.unsynced_change_time = Some(BackendSyncPoint::now());
        }
        self.did_write = true;

        Ok(())
    }
}

impl Drop for InMemoryWriter {
    fn drop(&mut self) {
        if self.did_write {
            self.completion_listener.notify_did_write(Instant::now());
        }
    }
}