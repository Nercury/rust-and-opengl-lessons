extern crate lzma;

use backend::{Writer, Reader, Backend, NotifyDidRead, NotifyDidWrite, BackendSyncPoint};
use self::lzma::LzmaReader;
use {ResourcePath, ResourcePathBuf};
use std::time::Instant;

pub struct Lzma<T> where T: Backend {
    inner: T,
}

impl<T> Backend for Lzma<T> where T: Backend {
    fn can_write(&self) -> bool {
        self.inner.can_write()
    }

    fn reader(&self, path: &ResourcePath, modification_time: Option<Instant>, completion_listener: Box<NotifyDidRead>) -> Option<Box<Reader>> {
        unimplemented!()
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.inner.exists(path)
    }

    fn writer(&self, path: &ResourcePath, completion_listener: Box<NotifyDidWrite>) -> Option<Box<Writer>> {
        unimplemented!()
    }

    fn notify_changes_synced(&self, point: BackendSyncPoint) {
        self.inner.notify_changes_synced(point);
    }

    fn new_changes(&self) -> Option<BackendSyncPoint> {
        self.inner.new_changes()
    }
}