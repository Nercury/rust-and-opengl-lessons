extern crate miniz_oxide as miniz;

use failure;
use backend::{Backend, BackendSyncPoint};
use std::io::{self};
use {ResourcePath, ResourcePathBuf, Error};

#[derive(Debug)]
pub struct Lzma<T> where T: Backend {
    inner: T,
    level: u8,
}

impl<T> Lzma<T> where T: Backend {
    pub fn new(inner: T, level: u8) -> Lzma<T> {
        Lzma {
            inner,
            level,
        }
    }
}

impl<T> Backend for Lzma<T> where T: Backend {
    fn can_write(&self) -> bool {
        self.inner.can_write()
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.inner.exists(path)
    }

    fn notify_changes_synced(&mut self, point: BackendSyncPoint) {
        self.inner.notify_changes_synced(point);
    }

    fn new_changes(&mut self) -> Option<BackendSyncPoint> {
        self.inner.new_changes()
    }

    fn read_into(&mut self, path: &ResourcePath, output: &mut io::Write) -> Result<(), Error> {
        let mut input_data = Vec::new();
        self.inner.read_into(path, &mut input_data)?;
        let output_data = self::miniz::inflate::decompress_to_vec_zlib(&mut input_data).map_err(write_error)?;
        output.write_all(&output_data[..])?;
        Ok(())
    }

    fn write_from(&mut self, path: &ResourcePath, buffer: &mut io::Read) -> Result<(), Error> {
        let mut input_data = Vec::new();
        buffer.read_to_end(&mut input_data)?;
        let output_data = self::miniz::deflate::compress_to_vec_zlib(&mut input_data, self.level);
        let mut cursor = io::Cursor::new(output_data);
        Ok(self.inner.write_from(path, &mut cursor)?)
    }
}

#[derive(Fail, Debug)]
pub enum MinizError {
    #[fail(display = "Miniz error {:?}", _0)]
    ErrorCode(self::miniz::inflate::TINFLStatus),
}

fn write_error(miniz_error: self::miniz::inflate::TINFLStatus) -> Error {
    Error::BackendFailedToWrite {
        path: ResourcePathBuf::from(String::from("")),
        inner: failure::Error::from(MinizError::ErrorCode(miniz_error)),
    }
}

#[cfg(test)]
mod test {
    use backend::{Backend, Lzma, InMemory};

    #[test]
    fn test_can_write_and_read() {
        let mut be = Lzma::new(InMemory::new(), 9);

        be.write("x".into(), b"hello world").unwrap();
        let result = be.read_vec("x".into()).unwrap();

        assert_eq!(b"hello world", &result[..]);
    }
}