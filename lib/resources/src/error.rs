use failure;
use std::io;
use ResourcePathBuf;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),
    #[fail(display = "Item not found")]
    NotFound,
    #[fail(display = "Backend can not write")]
    NotWritable,
    #[fail(display = "Failed to write {}, {}", path, inner)]
    BackendFailedToWrite {
        path: ResourcePathBuf,
        inner: failure::Error,
    },
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
            (Error::NotFound, Error::NotFound) => true,
            (Error::NotWritable, Error::NotWritable) => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }
}
