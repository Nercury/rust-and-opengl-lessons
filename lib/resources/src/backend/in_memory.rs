use backend::{Backend, BackendSyncPoint};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io;
use std::sync::{Arc, RwLock};
use twox_hash::XxHash;
use {Error, ResourcePath, ResourcePathBuf};

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
            shared: Arc::new(RwLock::new(Shared::new())),
        }
    }

    pub fn with<P: AsRef<ResourcePath>>(self, key: P, value: &[u8]) -> Self {
        self.shared
            .write()
            .expect("failed to lock InMemory for write")
            .insert(key.as_ref(), value);
        self
    }
}

impl Backend for InMemory {
    fn can_write(&self) -> bool {
        true
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.shared
            .read()
            .expect("failed to lock InMemory for read")
            .map
            .contains_key::<ResourcePath>(path.as_clean_str().as_ref())
    }

    fn notify_changes_synced(&mut self, point: BackendSyncPoint) {
        let mut shared_ref = self
            .shared
            .write()
            .expect("failed to lock InMemory for write");

        if shared_ref.unsynced_change_time == Some(point) {
            shared_ref.unsynced_change_time = None;
        }
    }

    fn new_changes(&mut self) -> Option<BackendSyncPoint> {
        self.shared
            .read()
            .expect("failed to lock InMemory for read")
            .unsynced_change_time
    }

    fn read_into(&mut self, path: &ResourcePath, output: &mut io::Write) -> Result<(), Error> {
        let shared = self
            .shared
            .read()
            .expect("failed to lock InMemory for read");
        let item_ref = match shared.map.get(path) {
            None => return Err(Error::NotFound),
            Some(val) => val,
        };
        output.write_all(&item_ref)?;
        Ok(())
    }

    fn write_from(&mut self, path: &ResourcePath, buffer: &mut io::Read) -> Result<(), Error> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut shared = self
            .shared
            .write()
            .expect("failed to lock InMemory for write");
        shared.map.insert(path.into(), data);
        shared.unsynced_change_time = Some(BackendSyncPoint::now());

        Ok(())
    }
}
