use backend::{Backend, BackendSyncPoint};
use path::{ResourcePath, ResourcePathBuf};
use slab::Slab;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::Instant;
use twox_hash::XxHash;

mod resource_metadata;

use self::resource_metadata::{ResourceMetadata, ResourceUserMetadata};

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

#[derive(Eq, PartialEq)]
pub enum InternalSyncPoint {
    Backend {
        backend_hash: u64,
        sync_point: BackendSyncPoint,
    },
    Everything {
        time: Instant,
    },
}

pub struct SharedResources {
    resource_metadata: Slab<ResourceMetadata>,
    path_resource_ids: HashMap<ResourcePathBuf, usize, BuildHasherDefault<XxHash>>,
    backends: BTreeMap<LoaderKey, Box<Backend>>,
    outdated_at: Option<Instant>,
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
            outdated_at: None,
        }
    }

    pub fn new_changes(&mut self) -> Option<InternalSyncPoint> {
        if let Some(instant) = self.outdated_at {
            return Some(InternalSyncPoint::Everything { time: instant });
        }
        for (key, backend) in self.backends.iter_mut() {
            if let Some(sync_point) = backend.new_changes() {
                return Some(InternalSyncPoint::Backend {
                    backend_hash: backend_hash(&key.id),
                    sync_point,
                });
            }
        }
        None
    }

    pub fn notify_changes_synced(&mut self, sync_point: InternalSyncPoint) {
        match sync_point {
            InternalSyncPoint::Everything { time } => if self.outdated_at == Some(time) {
                self.outdated_at = None;
            },
            InternalSyncPoint::Backend {
                backend_hash: bh,
                sync_point: sp,
            } => {
                for (key, backend) in self.backends.iter_mut() {
                    if backend_hash(&key.id) == bh {
                        backend.notify_changes_synced(sp);
                    }
                }
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
                self.path_resource_ids
                    .insert(ResourcePathBuf::from(clean_path_str), resource_id);

                UserKey {
                    resource_id,
                    user_id,
                }
            }
        }
    }

    /// Appends user to resource, the resource id must exist.
    pub fn append_resource_user(&mut self, resource_id: usize) -> UserKey {
        UserKey {
            resource_id,
            user_id: self
                .resource_metadata
                .get_mut(resource_id)
                .expect("expected resource_id to exist when appending new user")
                .new_user(),
        }
    }

    pub fn remove_resource_user(&mut self, key: UserKey) {
        let has_users = {
            if let Some(metadata) = self.resource_metadata.get_mut(key.resource_id) {
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

    pub fn get_path_user_metadata(&self, key: UserKey) -> Option<&ResourceUserMetadata> {
        self.resource_metadata
            .get(key.resource_id)
            .and_then(|path_metadata| path_metadata.get_user_metadata(key.user_id))
    }

    fn get_path_user_metadata_mut(&mut self, key: UserKey) -> Option<&mut ResourceUserMetadata> {
        self.resource_metadata
            .get_mut(key.resource_id)
            .and_then(|path_metadata| path_metadata.get_user_metadata_mut(key.user_id))
    }

    pub fn insert_loader<L: Backend + 'static>(
        &mut self,
        loader_id: &str,
        order: isize,
        backend: L,
    ) {
        let outdated_at = Instant::now();
        for (path, resource_id) in self.path_resource_ids.iter() {
            if backend.exists(&path) {
                if let Some(metadata) = self.resource_metadata.get_mut(*resource_id) {
                    metadata.everyone_should_reload(outdated_at);
                }
            }
        }
        self.backends.insert(
            LoaderKey {
                id: loader_id.into(),
                order,
            },
            Box::new(backend) as Box<Backend>,
        );
        if self.path_resource_ids.len() > 0 {
            self.outdated_at = Some(outdated_at);
        }
    }

    pub fn remove_loader(&mut self, loader_id: &str) {
        let outdated_at = Instant::now();
        let remove_keys: Vec<_> = self
            .backends
            .keys()
            .filter(|k| k.id == loader_id)
            .map(|k| k.clone())
            .collect();
        for removed_key in remove_keys {
            if let Some(removed_backend) = self.backends.remove(&removed_key) {
                for (path, resource_id) in self.path_resource_ids.iter() {
                    if removed_backend.exists(&path) {
                        if let Some(metadata) = self.resource_metadata.get_mut(*resource_id) {
                            metadata.everyone_should_reload(outdated_at);
                        }
                    }
                }
            }
        }
        if self.path_resource_ids.len() > 0 {
            self.outdated_at = Some(outdated_at);
        }
    }

    pub fn resource_backends(
        &mut self,
        key: UserKey,
    ) -> impl Iterator<Item = (&ResourcePath, Option<Instant>, &mut Box<Backend>)> {
        let path_with_modification_time =
            self.resource_metadata.get(key.resource_id).and_then(|m| {
                m.users
                    .get(key.user_id)
                    .map(|u| (m.path.as_ref(), u.outdated_at))
            });

        self.backends.iter_mut().rev().filter_map(move |(_, b)| {
            path_with_modification_time.map(move |(path, instant)| (path, instant, b))
        })
    }

    #[allow(dead_code)]
    pub fn get_resource_path_backend(
        &self,
        backend_id: &str,
        key: UserKey,
    ) -> Option<(&ResourcePath, Option<Instant>, &Box<Backend>)> {
        let path_with_modification_time =
            self.resource_metadata.get(key.resource_id).and_then(|m| {
                m.users
                    .get(key.user_id)
                    .map(|u| (m.path.as_ref(), u.outdated_at))
            });

        if let (Some((path, modification_time)), Some((_, backend))) = (
            path_with_modification_time,
            self.backends
                .iter()
                .filter(|(k, _)| &k.id == backend_id)
                .next(),
        ) {
            return Some((path, modification_time, backend));
        }

        None
    }

    pub fn get_resource_path(&self, key: UserKey) -> Option<&ResourcePath> {
        self.resource_metadata
            .get(key.resource_id)
            .map(|m| m.path.as_ref())
    }

    pub fn get_resource_path_backend_containing_resource(
        &self,
        key: UserKey,
    ) -> Option<(&ResourcePath, Option<Instant>, &Box<Backend>)> {
        let path_with_modification_time =
            self.resource_metadata.get(key.resource_id).and_then(|m| {
                m.users
                    .get(key.user_id)
                    .map(|u| (m.path.as_ref(), u.outdated_at))
            });

        if let Some((path, modification_time)) = path_with_modification_time {
            for backend in self.backends.values().rev() {
                if backend.exists(path) {
                    return Some((path, modification_time, backend));
                }
            }
        }

        None
    }

    pub fn notify_did_read(&mut self, key: UserKey, modified_time: Option<Instant>) {
        if let Some(metadata) = self.get_path_user_metadata_mut(key) {
            if metadata.outdated_at == modified_time {
                metadata.outdated_at = None;
            }
        }
    }

    pub fn notify_did_write(&mut self, key: UserKey, modified_time: Instant) {
        if let Some(metadata) = self.resource_metadata.get_mut(key.resource_id) {
            metadata.everyone_should_reload_except(key.user_id, modified_time)
        }
    }
}
