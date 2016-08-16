use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use super::Result;
use user_traits::User;
use std::sync::RwLock;

pub type ChannelThread = Sender<ChannelThreadMsg>;

pub type ChannelId = usize;

#[derive(Debug)]
pub enum ChannelThreadMsg {
    // INVARIANT: The Sender of this Join msg MUST place the ChannelId into a new ChannelEntry to ensure proper cleanup BEFORE any cloning to prevent double-free
    // it is impossible to handle this within the ChannelThread itself because it would create a circular reference. Even though it would work fine, it would prevent the DirectoryThread from automatically cleaning up
    Join(Sender<ChannelId>, User),
    Part(ChannelId, String, Option<String>),
    Privmsg(ChannelId, String, String),
    Who(ChannelId),
    Exit,
}

#[derive(Debug, Clone)]
pub struct ChannelEntry {
    arc: Arc<RwLock<StoredChannelId>>
}

unsafe impl Send for ChannelEntry{}

impl ChannelEntry {
    unsafe fn new(channel: Channel, id: ChannelId) -> Self {
        ChannelEntry{
            arc: Arc::new(RwLock::new(StoredChannelId{
                channel: channel,
                part_reason: None,
                mask: "".into(),
                id: id,
            })),
        }
    }

    pub fn update_mask(&self, mask: String) {
        let mut locked = self.arc.write().unwrap();
        locked.mask = mask.into();
    }

    pub fn part_reason(&self, reason: Option<String>) {
        let mut locked = self.arc.write().unwrap();
        locked.part_reason = reason;
    }

    pub fn privmsg(&self, mask: String, msg: String) -> Result<()>{
        let locked = self.arc.read().unwrap();
        try!(send!(locked.channel.thread, ChannelThreadMsg::Privmsg => (locked.id, mask, msg)));
        Ok(())
    }

    pub fn who(&self) -> Result<()> {
        let locked = self.arc.read().unwrap();
        try!(send!(locked.channel.thread, ChannelThreadMsg::Who => (locked.id)));
        Ok(())
    }
}

#[derive(Debug)]
struct StoredChannelId {
    channel: Channel,
    part_reason: Option<String>,
    mask: String,
    id: ChannelId,
}

impl Drop for StoredChannelId {
    fn drop(&mut self) {
        lprintln!("Dropping Channel ID -- {:?} -- {:?}", self.id, self.part_reason);
        self.channel.part(self.id, self.mask.clone(), self.part_reason.take()).unwrap();
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

    fn part(&self, id: ChannelId, mask: String, reason: Option<String>) -> Result<()> {
        try!(send!(self.thread, ChannelThreadMsg::Part => (id, mask, reason)));
        Ok(())
    }
}
