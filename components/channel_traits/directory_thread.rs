use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;
use super::ChannelThread;
use user_traits::UserThread;

pub type DirectoryThread = Sender<DirectoryThreadMsg>;

#[derive(Debug)]
pub enum DirectoryThreadMsg {
    GetChannels(Sender<Vec<ChannelThread>>),
    GetChannelByName(Sender<ChannelThread>, String),
    GetUserByName(Sender<UserThread>, String),
    Exit,
}

#[derive(Clone)]
pub struct Directory {
    thread: DirectoryThread,
}

impl Directory {
    pub fn new(thread: DirectoryThread) -> Self {
        Directory{ thread: thread }
    }
}


