use path::{ResourcePath};
use Error;
use std::io;
use std::time::Instant;

#[cfg(any(test, feature = "in_memory"))]
mod in_memory;
#[cfg(any(test, feature = "in_memory"))]
pub use self::in_memory::InMemory;

#[cfg(any(test, feature = "lzma"))]
mod lzma;
#[cfg(any(test, feature = "lzma"))]
pub use self::lzma::Lzma;

mod filesystem;
pub use self::filesystem::FileSystem;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct BackendSyncPoint {
    instant: Instant,
}

impl BackendSyncPoint {
    pub fn now() -> BackendSyncPoint {
        BackendSyncPoint {
            instant: Instant::now(),
        }
    }
}

pub trait NotifyDidRead: Send + Sync {
    fn notify_did_read(&self, modification_time: Option<Instant>);
}

pub trait NotifyDidWrite: Send + Sync {
    fn notify_did_write(&self, modification_time: Instant);
}

pub trait Backend: Send + Sync {
    fn can_write(&self) -> bool;
    fn reader(&self, path: &ResourcePath, modification_time: Option<Instant>, completion_listener: Box<NotifyDidRead>) -> Option<Box<Reader>>;
    fn exists(&self, path: &ResourcePath) -> bool;
    fn writer(&self, path: &ResourcePath, completion_listener: Box<NotifyDidWrite>) -> Option<Box<Writer>>;
    fn notify_changes_synced(&self, point: BackendSyncPoint);
    fn new_changes(&self) -> Option<BackendSyncPoint>;
}

pub trait Reader: Drop + Send + Sync {
    fn read_into(self: Box<Self>, output: &mut io::Write) -> Result<(), Error>;
    fn read_vec(self: Box<Self>) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.read_into(&mut output)?;
        Ok(output)
    }
}

pub trait Writer: Drop + Send + Sync {
    fn write_from(self: Box<Self>, buffer: &mut io::Read) -> Result<(), Error>;
    fn write(self: Box<Self>, mut value: &[u8]) -> Result<(), Error> {
        self.write_from(&mut value)?;
        Ok(())
    }
}