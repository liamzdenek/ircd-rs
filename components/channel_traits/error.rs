use std::result;
use util::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NickCollision,
    SendError(&'static str),
    RecvError(&'static str),
}

impl From<ChanError> for Error {
    fn from(err: ChanError) -> Error {
        match err {
            ChanError::SendError(err) => Error::SendError(err),
            ChanError::RecvError(err) => Error::RecvError(err),
        }
    }
}
