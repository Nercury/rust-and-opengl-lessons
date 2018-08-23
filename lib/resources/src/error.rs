use std::io;
use ResourcePathBuf;
use failure;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),
    #[fail(display = "Item {} has gone away", path)]
    ItemAtPathHasGoneAway { path: ResourcePathBuf },
    #[fail(display = "Failed to write {}, {}", path, inner)]
    BackendFailedToWrite { path: ResourcePathBuf, inner: failure::Error },
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
            (a, b) if a == b => true,
            _ => false,
        }
    }
}