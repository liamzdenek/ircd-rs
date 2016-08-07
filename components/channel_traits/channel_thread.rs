use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type ChannelThread = Sender<ChannelThreadMsg>;

#[derive(Debug)]
pub enum ChannelThreadMsg {
    Exit,
}

pub struct Channel {
    thread: ChannelThread,
}

impl Channel {
    
}
