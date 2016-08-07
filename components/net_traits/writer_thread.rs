use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type WriterThread = Sender<WriterThreadMsg>;

#[derive(Debug)]
pub enum WriterThreadMsg {
    Send(String)
}

pub struct Writer {
    thread: WriterThread,
}

impl Writer {
    pub fn new(thread: WriterThread) -> Self {
        Writer{ thread: thread }
    }

    pub fn write_raw(&self, msg: String) -> Result<()> {
        try!(send!(self.thread, WriterThreadMsg::Send => (msg)));
        Ok(())
    }
}
