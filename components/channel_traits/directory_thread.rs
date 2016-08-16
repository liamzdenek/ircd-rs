use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use util::*;
use super::Result;
use super::Channel;
use user_traits::User;

pub type DirectoryThread = Sender<DirectoryThreadMsg>;

pub type DirectoryId = u64;

#[derive(Debug)]
pub enum DirectoryThreadMsg {
    GetChannels(Sender<Vec<Channel>>),
    GetChannelByName(Sender<Channel>, String, String),
    GetUserByNick(Sender<Result<User>>, String),
    // INVARIANT: The Sender of this NewUser msg MUST place this Id into a new DirectoryEntry to ensure proper cleanup BEFORE any cloning to prevent double-free
    // it is impossible to handle this within the DirectoryThread itself because it would create a circular reference. even though it would work fine, it would  prevent the DirectoryThread from automatically cleaning up
    NewUser(Sender<DirectoryId>, User),
    UpdateNick(Sender<Result<()>>,DirectoryId, String),
    DestroyUser(DirectoryId),
    Exit,
}

#[derive(Debug,Clone)]
/*
the DirectoryEntry type is responsible for managing a DirectoryId and informing the Directory when this type goes out of scope. It internally stores an Arc to both the Directory and the ID, and when it is freed, it sends DirectoryThreadMsg::DestroyUser(id) to the DirectoryThread. Sync is implemented for this type so be careful when modifying it
*/
pub struct DirectoryEntry {
    id: Arc<StoredDirectoryId>
}

unsafe impl Send for DirectoryEntry{}

#[derive(Debug)]
struct StoredDirectoryId {
    directory: Directory,
    id: DirectoryId,
}

impl DirectoryEntry {
    unsafe fn new(directory: Directory, id: DirectoryId) -> Self {
        DirectoryEntry{
            id: Arc::new(StoredDirectoryId{
                directory: directory,
                id: id,
            })
        }
    }
    pub fn update_nick(&self, nick: String) -> Result<()> {
        let stored = self.id.clone();
        try!(try!(req_rep!(stored.directory.thread, DirectoryThreadMsg::UpdateNick => (stored.id, nick))));
        Ok(())
    }
}

impl Drop for StoredDirectoryId {
    fn drop(&mut self) {
        lprintln!("Dropping Directory ID -- {:?}", self.id);
        // we're about to wipe out the last reference, inform directory thread of its destruction
        self.directory.destroy_user(self.id).unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct Directory {
    thread: DirectoryThread,
}

impl Directory {
    pub fn new(thread: DirectoryThread) -> Self {
        Directory{ thread: thread }
    }

    pub fn new_user(&self, user: User) -> Result<DirectoryEntry> {
        unsafe{
            let id = try!(req_rep!(self.thread, DirectoryThreadMsg::NewUser => (user)));
            lprintln!("User got id: {:?}", id);
            Ok(DirectoryEntry::new(self.clone(), id))
        }
    }

    pub fn destroy_user(&self, id: DirectoryId) -> Result<()> {
        try!(send!(self.thread, DirectoryThreadMsg::DestroyUser => (id)));
        Ok(())
    }

    pub fn get_user_by_nick(&self, nick: String) -> Result<User> {
        try!(req_rep!(self.thread, DirectoryThreadMsg::GetUserByNick => (nick)))
    }

    pub fn get_channel_by_name(&self, name: String, nick: String) -> Result<Channel> {
        Ok(try!(req_rep!(self.thread, DirectoryThreadMsg::GetChannelByName => (name, nick))))
    }
}


