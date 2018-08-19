use path::{ResourcePath};
use Error;
use std::io;
use std::time::Instant;

mod in_memory;
pub use self::in_memory::InMemory;

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

pub trait NotifyDidRead {
    fn notify_did_read(&self);
}

pub trait NotifyDidWrite {
    fn notify_did_write(&self);
}

pub trait Backend {
    fn can_write(&self) -> bool;
    fn reader(&self, path: &ResourcePath, completion_listener: Box<NotifyDidRead>) -> Option<Box<Reader>>;
    fn exists(&self, path: &ResourcePath) -> bool;
    fn writer(&self, path: &ResourcePath, completion_listener: Box<NotifyDidWrite>) -> Option<Box<Writer>>;
    fn notify_changes_synced(&self, point: BackendSyncPoint);
    fn new_changes(&self) -> Option<BackendSyncPoint>;
}

pub trait Reader: Drop {
    fn read_into(&mut self, output: &mut io::Write) -> Result<(), Error>;
    fn read_vec(&mut self) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.read_into(&mut output)?;
        Ok(output)
    }
}

pub trait Writer: Drop {
    fn write_from(&mut self, buffer: &mut io::Read) -> Result<(), Error>;
    fn write(&mut self, mut value: &[u8]) -> Result<(), Error> {
        self.write_from(&mut value)?;
        Ok(())
    }
}