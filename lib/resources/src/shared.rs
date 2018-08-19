use std::io;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash;
use path::{ResourcePath, ResourcePathBuf};
use slab::Slab;
use backend::{Backend, BackendSyncPoint};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),
    #[fail(display = "Item {} has gone away", path)]
    ItemAtPathHasGoneAway { path: ResourcePathBuf },
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

impl ::std::cmp::PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::Io(_), Error::Io(_)) => true,
            (Error::ItemAtPathHasGoneAway { ref path }, Error::ItemAtPathHasGoneAway { path: ref path2 }) if path == path2 => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoaderKey {
    id: String,
    order: isize,
}

impl Ord for LoaderKey {
    fn cmp(&self, other: &LoaderKey) -> ::std::cmp::Ordering {
        match self.order.cmp(&other.order) {
            ::std::cmp::Ordering::Equal => self.id.cmp(&other.id),
            ordering => ordering,
        }
    }
}

impl PartialOrd for LoaderKey {
    fn partial_cmp(&self, other: &LoaderKey) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone)]
pub struct UserKey {
    pub resource_id: usize,
    user_id: usize,
}

pub struct ResourceUserMetadata {
    pub should_reload: bool,
}

pub struct ResourceMetadata {
    path: ResourcePathBuf,
    users: Slab<ResourceUserMetadata>,
}

impl ResourceMetadata {
    pub fn new(path: &ResourcePath) -> ResourceMetadata {
        ResourceMetadata {
            path: ResourcePathBuf::from(path),
            users: Slab::with_capacity(2),
        }
    }

    pub fn new_user(&mut self) -> usize {
        self.users.insert(ResourceUserMetadata {
            should_reload: false,
        })
    }

    pub fn remove_user(&mut self, id: usize) {
        self.users.remove(id);
        if self.users.len() > 8 && self.users.capacity() / self.users.len() > 2 {
            self.users.shrink_to_fit()
        }
    }

    pub fn get_user_metadata(&self, id: usize) -> Option<&ResourceUserMetadata> {
        self.users.get(id)
    }

    pub fn get_user_metadata_mut(&mut self, id: usize) -> Option<&mut ResourceUserMetadata> {
        self.users.get_mut(id)
    }

    pub fn has_users(&mut self) -> bool {
        self.users.len() > 0
    }

    pub fn should_reload_except(&mut self, id: usize) {
        for (user_id, user) in self.users.iter_mut() {
            user.should_reload = user_id != id;
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct SyncPoint {
    backend_hash: u64,
    sync_point: BackendSyncPoint,
}

pub struct SharedResources {
    resource_metadata: Slab<ResourceMetadata>,
    path_resource_ids: HashMap<ResourcePathBuf, usize, BuildHasherDefault<XxHash>>,
    backends: BTreeMap<LoaderKey, Box<Backend>>,
}

fn backend_hash(id: &str) -> u64 {
    use std::hash::Hasher;
    let mut hasher = XxHash::with_seed(8745287);
    hasher.write(id.as_bytes());
    hasher.finish()
}

impl SharedResources {
    pub fn new() -> SharedResources {
        SharedResources {
            resource_metadata: Slab::with_capacity(1024), // 1024 files is enough for everyone
            path_resource_ids: HashMap::default(),
            backends: BTreeMap::new(),
        }
    }

    pub fn new_changes(&mut self) -> Option<SyncPoint> {
        for (key, backend) in self.backends.iter_mut() {
            if let Some(sync_point) = backend.new_changes() {
                return Some(
                    SyncPoint {
                        backend_hash: backend_hash(&key.id),
                        sync_point,
                    }
                );
            }
        }
        None
    }

    pub fn notify_changes_synced(&mut self, sync_point: SyncPoint) {
        for (key, backend) in self.backends.iter_mut() {
            if backend_hash(&key.id) == sync_point.backend_hash {
                backend.notify_changes_synced(sync_point.sync_point);
            }
        }
    }

    pub fn new_resource_user<P: AsRef<ResourcePath>>(&mut self, path: P) -> UserKey {
        let clean_path_str: &ResourcePath = path.as_ref().as_clean_str().into();
        let maybe_id = self.path_resource_ids.get(clean_path_str).cloned();
        match maybe_id {
            Some(id) => self.append_resource_user(id),
            None => {
                let mut metadata = ResourceMetadata::new(clean_path_str);
                let user_id = metadata.new_user();
                let resource_id = self.resource_metadata.insert(metadata);
                self.path_resource_ids.insert(ResourcePathBuf::from(clean_path_str), resource_id);

                UserKey { resource_id, user_id }
            }
        }
    }

    /// Appends user to resource, the resource id must exist.
    pub fn append_resource_user(&mut self, resource_id: usize) -> UserKey {
        UserKey {
            resource_id,
            user_id: self.get_resource_metadata_mut(resource_id)
                .expect("expected resource_id to exist when appending new user")
                .new_user(),
        }
    }

    pub fn remove_resource_user(&mut self, key: UserKey) {
        let has_users = {
            if let Some(metadata) = self.get_resource_metadata_mut(key.resource_id) {
                metadata.remove_user(key.user_id);
                Some(metadata.has_users())
            } else {
                None
            }
        };

        if let Some(false) = has_users {
            let metadata = self.resource_metadata.remove(key.resource_id);
            self.path_resource_ids.remove(&metadata.path);
        }
    }

    fn get_resource_metadata(&self, id: usize) -> Option<&ResourceMetadata> {
        self.resource_metadata.get(id)
    }

    fn get_resource_metadata_mut(&mut self, id: usize) -> Option<&mut ResourceMetadata> {
        self.resource_metadata.get_mut(id)
    }

    pub fn get_path_user_metadata(&self, key: UserKey) -> Option<&ResourceUserMetadata> {
        self.resource_metadata.get(key.resource_id)
            .and_then(|path_metadata| path_metadata.get_user_metadata(key.user_id))
    }

    fn get_path_user_metadata_mut(&mut self, key: UserKey) -> Option<&mut ResourceUserMetadata> {
        self.resource_metadata.get_mut(key.resource_id)
            .and_then(|path_metadata| path_metadata.get_user_metadata_mut(key.user_id))
    }

    pub fn insert_loader<L: Backend + 'static>(&mut self, loader_id: &str, order: isize, loader: L) {
        self.backends.insert(
            LoaderKey { id: loader_id.into(), order },
            Box::new(loader) as Box<Backend>,
        );
    }

    pub fn get_resource_path_backend(&self, backend_id: &str, resource_id: usize) -> Option<(&ResourcePath, &Box<Backend>)> {
        let path = match self.get_resource_metadata(resource_id) {
            Some(ref m) => m.path.as_ref(),
            None => return None,
        };

        if let Some((_, backend)) = self.backends.iter().filter(|(k, _)| &k.id == backend_id).next() {
            return Some((path, backend));
        }

        None
    }

    pub fn get_resource_path_backend_containing_resource(&self, resource_id: usize) -> Option<(&ResourcePath, &Box<Backend>)> {
        let path = match self.get_resource_metadata(resource_id) {
            Some(ref m) => m.path.as_ref(),
            None => return None,
        };

        for backend in self.backends.values().rev() {
            if backend.exists(path) {
                return Some((path, backend));
            }
        }

        None
    }

    pub fn notify_did_read(&mut self, key: UserKey) {
        if let Some(metadata) = self.get_path_user_metadata_mut(key) {
            metadata.should_reload = false;
        }
    }

    pub fn notify_did_write(&mut self, key: UserKey) {
        if let Some(metadata) = self.get_resource_metadata_mut(key.resource_id) {
            metadata.should_reload_except(key.user_id)
        }
    }
}