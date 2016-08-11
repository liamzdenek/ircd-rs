use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use util::*;
use super::Result;
use user_traits::User;

pub type ChannelThread = Sender<ChannelThreadMsg>;

pub type ChannelId = usize;

#[derive(Debug)]
pub enum ChannelThreadMsg {
    // INVARIANT: The Sender of this Join msg MUST place the ChannelId into a new ChannelEntry to ensure proper cleanup BEFORE any cloning to prevent double-free
    // it is impossible to handle this within the ChannelThread itself because it would create a circular reference. Even though it would work fine, it would prevent the DirectoryThread from automatically cleaning up
    Join(Sender<ChannelId>, User),
    Part(ChannelId, Option<String>),
    Privmsg(ChannelId, String, String),
    Who(ChannelId),
    Exit,
}

#[derive(Debug, Clone)]
pub struct ChannelEntry {
    arc: Arc<StoredChannelId>
}

unsafe impl Send for ChannelEntry{}

impl ChannelEntry {
    unsafe fn new(channel: Channel, id: ChannelId) -> Self {
        ChannelEntry{
            arc: Arc::new(StoredChannelId{
                channel: channel,
                part_reason: None,
                id: id,
            }),
        }
    }

    pub fn privmsg(&self, mask: String, msg: String) -> Result<()>{
        try!(send!(self.arc.channel.thread, ChannelThreadMsg::Privmsg => (self.arc.id, mask, msg)));
        Ok(())
    }

    pub fn who(&self) -> Result<()> {
        try!(send!(self.arc.channel.thread, ChannelThreadMsg::Who => (self.arc.id)));
        Ok(())
    }
}

#[derive(Debug)]
struct StoredChannelId {
    channel: Channel,
    part_reason: Option<String>,
    id: ChannelId,
}

impl Drop for StoredChannelId {
    fn drop(&mut self) {
        println!("Dropping Channel ID -- {:?}", self.id);
        self.channel.part(self.id, self.part_reason.take()).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    thread: ChannelThread,
}

impl Channel {
    pub fn new(thread: ChannelThread) -> Self {
        Channel{ thread: thread }
    }

    pub fn join(&self, user: User) -> Result<ChannelEntry> {
        unsafe{
            let id = try!(req_rep!(self.thread, ChannelThreadMsg::Join => (user)));
            Ok(ChannelEntry::new(self.clone(), id))
        }
    }

    fn part(&self, id: ChannelId, reason: Option<String>) -> Result<()> {
        try!(send!(self.thread, ChannelThreadMsg::Part => (id, reason)));
        Ok(())
    }
}
