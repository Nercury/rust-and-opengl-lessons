use crate::path::ResourcePath;
use std::io;
use std::time::Instant;
use crate::Error;

#[cfg(any(test, feature = "backend_in_memory"))]
mod in_memory;
#[cfg(any(test, feature = "backend_in_memory"))]
pub use self::in_memory::InMemory;

#[cfg(any(test, feature = "backend_miniz"))]
mod miniz;
#[cfg(any(test, feature = "backend_miniz"))]
pub use self::miniz::Miniz;

#[cfg(any(test, feature = "backend_filesystem"))]
mod filesystem;
#[cfg(any(test, feature = "backend_filesystem"))]
pub use self::filesystem::FileSystem;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

pub trait Backend: Send + Sync {
    fn can_write(&self) -> bool;
    fn exists(&self, path: &ResourcePath) -> bool;

    fn notify_changes_synced(&mut self, point: BackendSyncPoint);
    fn new_changes(&mut self) -> Option<BackendSyncPoint>;

    fn read_into(&mut self, path: &ResourcePath, output: &mut io::Write) -> Result<(), Error>;
    fn read_vec(&mut self, path: &ResourcePath) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.read_into(path, &mut output)?;
        Ok(output)
    }

    fn write_from(&mut self, path: &ResourcePath, buffer: &mut io::Read) -> Result<(), Error>;
    fn write(&mut self, path: &ResourcePath, mut value: &[u8]) -> Result<(), Error> {
        self.write_from(path, &mut value)?;
        Ok(())
    }
}
