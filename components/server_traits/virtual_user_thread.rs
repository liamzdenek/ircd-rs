use super::Result;
use std::sync::mpsc::{Sender};

pub type VirtualUserThread = Sender<VirtualUserThreadMsg>;

#[derive(Debug)]
pub enum VirtualUserThreadMsg {
    Join(String), // channel
    Part(String), // channel
    PrivmsgChan(String, String), // channel, msg
    Exit,
}

pub struct VirtualUser {
    thread: VirtualUserThread,
}

impl VirtualUser {
    pub fn new(t: VirtualUserThread) -> Self {
        VirtualUser{
            thread: t,
        }
    }

    pub fn join(&self, chan: String) -> Result<()> {
        try!(send!(self.thread, VirtualUserThreadMsg::Join => (chan)));
        Ok(())
    }

    pub fn part(&self, chan: String) -> Result<()> {
        try!(send!(self.thread, VirtualUserThreadMsg::Part => (chan)));
        Ok(())
    }

    pub fn privmsg_chan(&self, chan: String, msg: String) -> Result<()> {
        try!(send!(self.thread, VirtualUserThreadMsg::PrivmsgChan => (chan, msg)));
        Ok(())
    }
}
