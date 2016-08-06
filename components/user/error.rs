use std::result;
use std::io::Error as IoError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    MalformedString,
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::IoError(err)
    }
}
