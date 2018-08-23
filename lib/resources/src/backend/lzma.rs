extern crate lzma;

use backend::{Writer, Reader, Backend, NotifyDidRead, NotifyDidWrite, BackendSyncPoint};
use self::lzma::{LzmaReader};
use self::lzma::error::{LzmaError};
use std::io::{Write, Read};
use {ResourcePath, ResourcePathBuf, Error};
use std::time::Instant;

#[derive(Debug)]
pub struct Lzma<T> where T: Backend {
    inner: T,
    preset: u32,
}

impl<T> Lzma<T> where T: Backend {
    pub fn new(inner: T, preset: u32) -> Lzma<T> {
        Lzma {
            inner,
            preset
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
        let preset = self.preset;
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
        unimplemented!("read_into")
    }
}

impl Drop for LzmaBackendReader {
    fn drop(&mut self) {
    }
}

struct LzmaBackendWriter {
    inner: Box<Writer>,
    preset: u32,
}

impl Writer for LzmaBackendWriter {
    fn write_from(self: Box<Self>, buffer: &mut Read) -> Result<(), Error> {
        let mut compressor = LzmaReader::new_compressor(buffer, self.preset).map_err(write_error)?;
        Ok(self.inner.write_from(&mut compressor)?)
    }
}

fn write_error(lzma_error: LzmaError) -> Error {
    Error::BackendFailedToWrite {
        path: ResourcePathBuf::from(String::from("")),
        inner: Box::new(lzma_error).into()
    }
}



#[cfg(test)]
mod test {
    use std::time::Instant;
    use backend::{Backend, NotifyDidWrite, Lzma, InMemory};
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

//        {
//            let reader = be
//                .reader("x".into(), TestListener::new_boxed() as Box<NotifyDidWrite>)
//                .unwrap();
//            writer.write(b"hello world").unwrap();
//        }
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
}