#[macro_use]
extern crate failure;
extern crate slab;
extern crate twox_hash;

mod path;

pub use self::path::{ResourcePath, ResourcePathBuf};

mod shared;

use self::shared::{SharedResources, UserKey, InternalSyncPoint};

pub mod backend;
use self::backend::{NotifyDidRead, NotifyDidWrite};

mod error;
pub use self::error::Error;

use std::time::Instant;
use std::sync::Arc;
use std::sync::RwLock;

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

    pub fn loaded_from<L: backend::Backend + 'static>(self, loader_id: &str, order: isize, backend: L) -> Resources {
        self.insert_loader(loader_id, order, backend);
        self
    }

    pub fn insert_loader<L: backend::Backend + 'static>(&self, loader_id: &str, order: isize, backend: L) {
        let mut resources = self.shared.write()
            .expect("failed to lock for write");
        resources.insert_loader(loader_id, order, backend);
    }

    pub fn remove_loader(&self, loader_id: &str) {
        let mut resources = self.shared.write()
            .expect("failed to lock for write");
        resources.remove_loader(loader_id);
    }

    pub fn resource<P: AsRef<ResourcePath>>(&self, path: P) -> Resource {
        Resource {
            shared: self.shared.clone(),
            key: self.shared.write()
                .expect("failed to lock for write")
                .new_resource_user(path),
        }
    }

    pub fn new_changes(&self) -> Option<SyncPoint> {
        self.shared.write()
            .expect("failed to lock for write")
            .new_changes()
            .map(|p| SyncPoint(p))
    }

    pub fn notify_changes_synced(&self, sync_point: SyncPoint) {
        self.shared.write()
            .expect("failed to lock for write")
            .notify_changes_synced(sync_point.0)
    }
}

struct NotifyDidReadForResources {
    shared: Arc<RwLock<SharedResources>>,
    key: UserKey,
}

impl NotifyDidRead for NotifyDidReadForResources {
    fn notify_did_read(&self, modification_time: Option<Instant>) {
        self.shared.write()
            .expect("failed to lock for write")
            .notify_did_read(self.key, modification_time)
    }
}

struct NotifyDidWriteForResources {
    shared: Arc<RwLock<SharedResources>>,
    key: UserKey,
}

impl NotifyDidWrite for NotifyDidWriteForResources {
    fn notify_did_write(&self, modification_time: Instant) {
        self.shared.write()
            .expect("failed to lock for write")
            .notify_did_write(self.key, modification_time)
    }
}

pub struct Resource {
    shared: Arc<RwLock<SharedResources>>,
    key: UserKey,
}

impl Resource {
    pub fn any_reader(&self) -> Option<Box<backend::Reader>> {
        let shared_ref = &self.shared;
        let key_ref = &self.key;

        let resources = shared_ref.read()
            .expect("failed to lock for read");

        resources
            .get_resource_path_backend_containing_resource(self.key)
            .and_then(|(path, modification_time, backend)|
                backend.reader(
                    path,
                    modification_time,
                    Box::new(NotifyDidReadForResources {
                        shared: shared_ref.clone(),
                        key: *key_ref,
                    }) as Box<NotifyDidRead>,
                )
            )
    }

    pub fn exact_reader(&self, backend_id: &str) -> Option<Box<backend::Reader>> {
        let shared_ref = &self.shared;
        let key_ref = &self.key;

        let resources = shared_ref.read()
            .expect("failed to lock for read");

        resources
            .get_resource_path_backend(backend_id, self.key)
            .and_then(|(path, modification_time, backend)|
                backend.reader(
                    path,
                    modification_time,
                    Box::new(NotifyDidReadForResources {
                        shared: shared_ref.clone(),
                        key: *key_ref,
                    }) as Box<NotifyDidRead>,
                )
            )
    }

    pub fn exact_writer(&self, backend_id: &str) -> Option<Box<backend::Writer>> {
        let shared_ref = &self.shared;
        let key_ref = &self.key;

        let resources = shared_ref.read()
            .expect("failed to lock for read");

        resources
            .get_resource_path_backend(backend_id, self.key)
            .and_then(|(path, _, backend)|
                if backend.can_write() {
                    backend.writer(
                        path,
                        Box::new(NotifyDidWriteForResources {
                            shared: shared_ref.clone(),
                            key: *key_ref,
                        }) as Box<NotifyDidWrite>,
                    )
                } else {
                    None
                }
            )
    }

    pub fn is_modified(&self) -> bool {
        let resources = self.shared.read()
            .expect("failed to lock for read");
        resources.get_path_user_metadata(self.key)
            .map(|m| m.outdated_at.is_some())
            .unwrap_or(false)
    }
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        let new_key = {
            let mut resources = self.shared.write()
                .expect("failed to lock for write");
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
        let mut resources = self.shared.write()
            .expect("failed to lock for write");
        resources.remove_resource_user(self.key);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with_no_loaders_should_have_no_reader() {
        let res = Resources::new();
        let reader = res.resource("a").any_reader();
        assert!(reader.is_none());
    }

    #[test]
    fn should_read_value() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        assert_eq!(
            &res
                .resource("name")
                .any_reader()
                .expect(r#"path "name" should exist"#)
                .read_vec()
                .unwrap(),
            b"hello"
        );
    }

    #[test]
    fn there_should_be_no_changes_and_resources_should_not_be_modified_at_start() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

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
    fn writing_resource_should_produce_change_sync_point_and_other_resource_proxies_should_see_it_as_modified() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");
        let resource_proxy_clone_a = resource_proxy_a.clone();
        let resource_proxy_clone_b = resource_proxy_b.clone();

        assert!(
            resource_proxy_b.exact_writer("a").expect(r#"path "name" should exist"#)
                .write(b"world").is_ok()
        );

        assert!(res.new_changes().is_some());

        assert!(resource_proxy_a.is_modified());
        assert!(!resource_proxy_b.is_modified(), "the most recent written item is assumed to be up to date");
        assert!(resource_proxy_clone_a.is_modified());
        assert!(resource_proxy_clone_b.is_modified());
    }

    #[test]
    fn notifying_changes_synced_should_clear_syn_point() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");

        resource_proxy_b.exact_writer("a").expect(r#"path "name" should exist"#)
            .write(b"world").unwrap();

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        res.notify_changes_synced(point);

        assert!(resource_proxy_a.is_modified(), "resources remain marked as modified until read");
        assert!(!resource_proxy_b.is_modified(), "last written resource looses modified state");
        assert!(res.new_changes().is_none());
    }

    #[test]
    fn notifying_changes_synced_should_not_clear_syn_point_if_there_were_new_writes() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        let resource_proxy_a = res.resource("name");
        let resource_proxy_b = res.resource("name");

        resource_proxy_b.exact_writer("a").expect(r#"path "name" should exist"#)
            .write(b"world").unwrap();

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        resource_proxy_a.exact_writer("a").expect(r#"path "name" should exist"#)
            .write(b"world2").unwrap();

        res.notify_changes_synced(point);

        assert!(resource_proxy_b.is_modified(), "resources remain marked as modified until read");
        assert!(!resource_proxy_a.is_modified(), "last written resource looses modified state");
        assert!(res.new_changes().is_some());
    }

    #[test]
    fn removing_the_loader_should_invalidate_resource() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        let resource_proxy_a = res.resource("name");

        res.remove_loader("a");

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        assert!(resource_proxy_a.is_modified(), "removed loader should trigger modified flag on resource");
        res.notify_changes_synced(point);

        assert!(res.new_changes().is_none());
    }

    #[test]
    fn adding_the_loader_should_override_resource_and_invalidate_it() {
        let res = Resources::new()
            .loaded_from(
                "a", 0,
                backend::InMemory::new()
                    .with("name", b"hello"),
            );

        let resource_proxy_a = res.resource("name");

        res.insert_loader("b", 1,
                          backend::InMemory::new()
                              .with("name", b"world"));

        assert!(res.new_changes().is_some());
        let point = res.new_changes().unwrap();

        assert!(resource_proxy_a.is_modified(), "adding loader should trigger modified flag on resource");

        assert_eq!(
            &resource_proxy_a.any_reader().expect(r#"path "name" should exist"#)
                .read_vec().unwrap(),
            b"world"
        );
        assert!(!resource_proxy_a.is_modified(), "reading resouce should mark it read");
        res.notify_changes_synced(point);

        assert!(res.new_changes().is_none());
    }
}