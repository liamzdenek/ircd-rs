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
    Part(ChannelId),
    Privmsg(String, String),
    Exit,
}

#[derive(Debug)]
pub struct ChannelEntry {
    id: Arc<StoredChannelId>
}

unsafe impl Send for ChannelEntry{}

impl ChannelEntry {
    unsafe fn new(channel: Channel, id: ChannelId) -> Self {
        ChannelEntry{
            id: Arc::new(StoredChannelId{
                channel: channel,
                id: id,
            }),
        }
    }
}

#[derive(Debug)]
struct StoredChannelId {
    channel: Channel,
    id: ChannelId,
}

impl Drop for StoredChannelId {
    fn drop(&mut self) {
        println!("Dropping Channel ID -- {:?}", self.id);
        self.channel.part(self.id).unwrap();
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

    fn part(&self, id: ChannelId) -> Result<()> {
        try!(send!(self.thread, ChannelThreadMsg::Part => (id)));
        Ok(())
    }
}
