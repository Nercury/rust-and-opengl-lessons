#[macro_use]
extern crate failure;
extern crate slab;
extern crate twox_hash;
#[macro_use]
extern crate log;

#[cfg(feature = "backend_filesystem_watch")]
extern crate notify;

mod path;

pub use self::path::{ResourcePath, ResourcePathBuf};

mod shared;

use self::shared::{InternalSyncPoint, SharedResources, UserKey};

pub mod backend;

mod error;
pub use self::error::Error;

use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

pub struct SyncPoint(InternalSyncPoint);

#[derive(Clone)]
pub struct Resources {
    shared: Arc<RwLock<SharedResources>>,
}

impl Resources {
    pub fn new() -> Resources {
        Resources {
            shared: Arc::new(RwLock::new(SharedResources::new())),
        }
    }

    pub fn loaded_from<L: backend::Backend + 'static>(
        self,
        loader_id: &str,
        order: isize,
        backend: L,
    ) -> Resources {
        self.insert_loader(loader_id, order, backend);
        self
    }

    pub fn insert_loader<L: backend::Backend + 'static>(
        &self,
        loader_id: &str,
        order: isize,
        backend: L,
    ) {
        let mut resources = self.shared.write().expect("failed to lock for write");
        resources.insert_loader(loader_id, order, backend);
    }

    pub fn remove_loader(&self, loader_id: &str) {
        let mut resources = self.shared.write().expect("failed to lock for write");
        resources.remove_loader(loader_id);
    }

    pub fn resource<P: AsRef<ResourcePath>>(&self, path: P) -> Resource {
        Resource {
            shared: self.shared.clone(),
            key: self
                .shared
                .write()
                .expect("failed to lock for write")
                .new_resource_user(path),
        }
    }

    pub fn new_changes(&self) -> Option<SyncPoint> {
        self.shared
            .write()
            .expect("failed to lock for write")
            .new_changes()
            .map(|p| SyncPoint(p))
    }

    pub fn notify_changes_synced(&self, sync_point: SyncPoint) {
        self.shared
            .write()
            .expect("failed to lock for write")
            .notify_changes_synced(sync_point.0)
    }
}

pub struct Resource {
    shared: Arc<RwLock<SharedResources>>,
    key: UserKey,
}

impl Resource {
    pub fn name(&self) -> String {
        let shared_ref = &self.shared;
        let resources = shared_ref.read().expect("failed to lock for read");

        resources
            .get_resource_path(self.key)
            .map(|p| p.to_string())
            .expect("expected resource to have access to the name")
    }

    /// Check if this resource exists.
    ///
    /// This unreliable command can tell if at least one backend can return the resource at this moment.
    /// Not that the next moment the resource can be gone.
    pub fn exists(&self) -> bool {
        let shared_ref = &self.shared;
        let resources = shared_ref.read().expect("failed to lock for read");

        resources
            .get_resource_path_backend_containing_resource(self.key)
            .map(|(path, _, b)| b.exists(path))
            .unwrap_or(false)
    }

    /// Read value from the backend that has highest order number and contains the resource.
    pub fn get(&self) -> Result<Vec<u8>, Error> {
        let shared_ref = &self.shared;
        let mut resources = shared_ref.write().expect("failed to lock for write");

        let mut did_read = None;

        {
            for (path, modification_time, backend) in resources.resource_backends(self.key) {
                match backend.read_vec(path) {
                    Ok(result) => {
                        did_read = Some((modification_time, result));
                        break;
                    }
                    Err(Error::NotFound) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        if let Some((modification_time, result)) = did_read {
            resources.notify_did_read(self.key, modification_time);
            return Ok(result);
        }

        Err(Error::NotFound)
    }

    /// Write value to the backend that has highest order number and can write.
    pub fn write(&self, data: &[u8]) -> Result<(), Error> {
        let shared_ref = &self.shared;
        let mut resources = shared_ref.write().expect("failed to lock for write");

        let mut did_write = false;

        {
            for (path, _, backend) in resources.resource_backends(self.key) {
                match backend.write(path, data) {
                    Ok(()) => {
                        did_write = true;
                        break;
                    }
                    Err(Error::NotWritable) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        if did_write {
            resources.notify_did_write(self.key, Instant::now());
            return Ok(());
        }

        Err(Error::NotWritable)
    }

    pub fn is_modified(&self) -> bool {
        let resources = self.shared.read().expect("failed to lock for read");
        resources
            .get_path_user_metadata(self.key)
            .map(|m| m.outdated_at.is_some())
            .unwrap_or(false)
    }
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        let new_key = {
            let mut resources = self.shared.write().expect("failed to lock for write");
            resources.append_resource_user(self.key.resource_id)
        };

        Resource {
            shared: self.shared.clone(),
            key: new_key,
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        let mut resources = self.shared.write().expect("failed to lock for write");
        resources.remove_resource_user(self.key);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with_no_loaders_should_have_no_reader() {
        let res = Resources::new();
        assert!(!res.resource("a").exists());
    }

    #[test]
    fn should_read_value() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        assert_eq!(&res.resource("name").get().unwrap(), b"hello");
    }

    #[test]
    fn there_should_be_no_changes_and_resources_should_not_be_modified_at_start() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        assert!(res.new_changes().is_none());

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");
        let resource_proxy_clone_a = resource_proxy_a.clone();
        let resource_proxy_clone_b = resource_proxy_b.clone();

        assert!(res.new_changes().is_none());

        assert!(!resource_proxy_a.is_modified());
        assert!(!resource_proxy_b.is_modified());
        assert!(!resource_proxy_clone_a.is_modified());
        assert!(!resource_proxy_clone_b.is_modified());
    }

    #[test]
    fn writing_resource_should_produce_change_sync_point_and_other_resource_proxies_should_see_it_as_modified(
) {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");
        let resource_proxy_clone_a = resource_proxy_a.clone();
        let resource_proxy_clone_b = resource_proxy_b.clone();

        assert!(resource_proxy_b.write(b"world").is_ok());

        assert!(res.new_changes().is_some());

        assert!(resource_proxy_a.is_modified());
        assert!(
            !resource_proxy_b.is_modified(),
            "the most recent written item is assumed to be up to date"
        );
        assert!(resource_proxy_clone_a.is_modified());
        assert!(resource_proxy_clone_b.is_modified());
    }

    #[test]
    fn notifying_changes_synced_should_clear_syn_point() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");

        resource_proxy_b.write(b"world").unwrap();

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        res.notify_changes_synced(point);

        assert!(
            resource_proxy_a.is_modified(),
            "resources remain marked as modified until read"
        );
        assert!(
            !resource_proxy_b.is_modified(),
            "last written resource looses modified state"
        );
        assert!(res.new_changes().is_none());
    }

    #[test]
    fn notifying_changes_synced_should_not_clear_syn_point_if_there_were_new_writes() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");

        resource_proxy_b.write(b"world").unwrap();

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        resource_proxy_a.write(b"world2").unwrap();

        res.notify_changes_synced(point);

        assert!(
            resource_proxy_b.is_modified(),
            "resources remain marked as modified until read"
        );
        assert!(
            !resource_proxy_a.is_modified(),
            "last written resource looses modified state"
        );
        assert!(res.new_changes().is_some());
    }

    #[test]
    fn removing_the_loader_should_invalidate_resource() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        let resource_proxy_a = res.resource("name");

        res.remove_loader("a");

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        assert!(
            resource_proxy_a.is_modified(),
            "removed loader should trigger modified flag on resource"
        );
        res.notify_changes_synced(point);

        assert!(res.new_changes().is_none());
    }

    #[test]
    fn adding_the_loader_should_override_resource_and_invalidate_it() {
        let res =
            Resources::new().loaded_from("a", 0, backend::InMemory::new().with("name", b"hello"));

        let resource_proxy_a = res.resource("name");

        res.insert_loader("b", 1, backend::InMemory::new().with("name", b"world"));

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        assert!(
            resource_proxy_a.is_modified(),
            "adding loader should trigger modified flag on resource"
        );

        assert_eq!(&resource_proxy_a.get().unwrap(), b"world");
        assert!(
            !resource_proxy_a.is_modified(),
            "reading resouce should mark it read"
        );
        res.notify_changes_synced(point);

        assert!(res.new_changes().is_none());
    }
}
