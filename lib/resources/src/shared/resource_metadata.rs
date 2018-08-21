use {ResourcePath, ResourcePathBuf};
use slab::Slab;
use std::time::Instant;

/// Information about the latest resource update.
///
/// If it is none, there are no updates, otherwise it contains a timestamp of the latest update.
pub struct ResourceUserMetadata {
    pub outdated_at: Option<Instant>,
}

/// Shared information about the resource.
///
/// Each resource can be owned by multiple proxies (called `Resource`). In that case, every proxy
/// gets an identifier from the `users` slab, and can check for resource updates in
/// `ResourceUserMetadata`.
pub struct ResourceMetadata {
    pub path: ResourcePathBuf,
    pub users: Slab<ResourceUserMetadata>,
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
            outdated_at: None,
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

    pub fn everyone_should_reload_except(&mut self, id: usize, outdated_at: Instant) {
        for (user_id, user) in self.users.iter_mut() {
            user.outdated_at = if user_id != id { Some(outdated_at) } else { None };
        }
    }

    pub fn everyone_should_reload(&mut self, outdated_at: Instant) {
        for (_, user) in self.users.iter_mut() {
            user.outdated_at = Some(outdated_at);
        }
    }
}