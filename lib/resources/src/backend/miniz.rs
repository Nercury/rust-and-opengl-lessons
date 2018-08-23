extern crate miniz_oxide as miniz;

use failure;
use backend::{Writer, Reader, Backend, NotifyDidRead, NotifyDidWrite, BackendSyncPoint};
use std::io::{self, Write, Read};
use {ResourcePath, ResourcePathBuf, Error};
use std::time::Instant;

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

    fn reader(&self, path: &ResourcePath, modification_time: Option<Instant>, completion_listener: Box<NotifyDidRead>) -> Option<Box<Reader>> {
        self.inner.reader(path, modification_time, completion_listener)
            .map(|inner| Box::new(LzmaBackendReader { inner }) as Box<Reader>)
    }

    fn exists(&self, path: &ResourcePath) -> bool {
        self.inner.exists(path)
    }

    fn writer(&self, path: &ResourcePath, completion_listener: Box<NotifyDidWrite>) -> Option<Box<Writer>> {
        let preset = self.level;
        self.inner.writer(path, completion_listener)
            .map(|inner| Box::new(LzmaBackendWriter { inner, preset }) as Box<Writer>)
    }

    fn notify_changes_synced(&self, point: BackendSyncPoint) {
        self.inner.notify_changes_synced(point);
    }

    fn new_changes(&self) -> Option<BackendSyncPoint> {
        self.inner.new_changes()
    }
}

struct LzmaBackendReader {
    inner: Box<Reader>,
}

impl Reader for LzmaBackendReader {
    fn read_into(self: Box<Self>, output: &mut Write) -> Result<(), Error> {
        let mut input_data = Vec::new();
        self.inner.read_into(&mut input_data)?;
        let output_data = self::miniz::inflate::decompress_to_vec_zlib(&mut input_data).map_err(write_error)?;
        output.write_all(&output_data[..])?;
        Ok(())
    }
}

struct LzmaBackendWriter {
    inner: Box<Writer>,
    preset: u8,
}

#[derive(Fail, Debug)]
pub enum MinizError {
    #[fail(display = "Miniz error {:?}", _0)]
    ErrorCode(self::miniz::inflate::TINFLStatus),
}

impl Writer for LzmaBackendWriter {
    fn write_from(self: Box<Self>, buffer: &mut Read) -> Result<(), Error> {
        let mut input_data = Vec::new();
        buffer.read_to_end(&mut input_data)?;
        let output_data = self::miniz::deflate::compress_to_vec_zlib(&mut input_data, self.preset);
        let mut cursor = io::Cursor::new(output_data);
        Ok(self.inner.write_from(&mut cursor)?)
    }
}

fn write_error(miniz_error: self::miniz::inflate::TINFLStatus) -> Error {
    Error::BackendFailedToWrite {
        path: ResourcePathBuf::from(String::from("")),
        inner: failure::Error::from(MinizError::ErrorCode(miniz_error))
    }
}



#[cfg(test)]
mod test {
    use std::time::Instant;
    use backend::{Backend, NotifyDidWrite, NotifyDidRead, Lzma, InMemory};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_can_write_and_read() {
        let be = Lzma::new(InMemory::new(), 9);

        let modification_time = Arc::new(Mutex::new(None));
        let listener = TestListener::new_boxed(modification_time.clone()) as Box<NotifyDidWrite>;

        {
            let writer = be
                .writer("x".into(), listener)
                .unwrap();
            writer.write(b"hello world").unwrap();
        }

        let listener = TestListener::new_boxed(modification_time.clone()) as Box<NotifyDidRead>;
        let write_time = *modification_time.lock().unwrap();

        let result = {
            let reader = be
                .reader("x".into(), write_time, listener)
                .unwrap();
            reader.read_vec().unwrap()
        };
    }

    struct TestListener {
        pub modification_time: Arc<Mutex<Option<Instant>>>,
    }

    impl TestListener {
        pub fn new_boxed(result: Arc<Mutex<Option<Instant>>>) -> Box<TestListener> {
            Box::new(
                TestListener {
                    modification_time: result,
                }
            )
        }
    }

    impl NotifyDidWrite for TestListener {
        fn notify_did_write(&self, modification_time: Instant) {
            *self.modification_time.lock().unwrap() = Some(modification_time);
        }
    }

    impl NotifyDidRead for TestListener {
        fn notify_did_read(&self, modification_time: Option<Instant>) {
            *self.modification_time.lock().unwrap() = modification_time;
        }
    }
}