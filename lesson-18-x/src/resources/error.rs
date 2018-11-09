use super::obj;
use image;
use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] io::Error),
    #[fail(display = "Failed to read CString from file that contains 0")]
    FileContainsNil,
    #[fail(display = "Failed get executable path")]
    FailedToGetExePath,
    #[fail(display = "Failed to load image")]
    FailedToLoadImage(#[cause] image::ImageError),
    #[fail(display = "Image {} is not RGBA", name)]
    ImageIsNotRgba { name: String },
    #[fail(display = "Failed to load {} obj file", name)]
    FailedToLoadObj {
        name: String,
        #[cause]
        inner: obj::Error,
    },
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

impl From<image::ImageError> for Error {
    fn from(other: image::ImageError) -> Self {
        Error::FailedToLoadImage(other)
    }
}
