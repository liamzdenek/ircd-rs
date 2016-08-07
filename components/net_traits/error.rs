use std::result;
use std::io::Error as IoError;
use util::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    SendError(&'static str),
    RecvError(&'static str),
    IoError(IoError),
    MalformedString,
    UserError,
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::IoError(err)
    }
}

impl From<ChanError> for Error {
    fn from(err: ChanError) -> Error {
        match err {
            ChanError::SendError(err) => Error::SendError(err),
            ChanError::RecvError(err) => Error::RecvError(err),
        }
    }
}
