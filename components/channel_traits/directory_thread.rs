use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use util::*;
use super::Result;
use super::ChannelThread;
use user_traits::User;

pub type DirectoryThread = Sender<DirectoryThreadMsg>;

pub type DirectoryId = u64;

#[derive(Debug)]
pub enum DirectoryThreadMsg {
    GetChannels(Sender<Vec<ChannelThread>>),
    GetChannelByName(Sender<ChannelThread>, String),
    GetUserByName(Sender<User>, String),
    // INVARIANT: The Sender of this NewUser msg MUST place this Id into a new UserEntry to ensure proper cleanup BEFORE any cloning to prevent double-free
    // it is impossible to handle this within the DirectoryThread itself because it would create a circular reference. even though it would work fine, it would  prevent the DirectoryThread from automatically cleaning up
    NewUser(Sender<DirectoryId>, User),
    UpdateNick(Sender<Result<()>>,DirectoryId, String),
    DestroyUser(DirectoryId),
    Exit,
}

#[derive(Clone)]
/*
the UserEntry type is responsible for managing a DirectoryId and informing the Directory when this type goes out of scope. It internally stores an Arc to both the Directory and the ID, and when it is freed, it sends DirectoryThreadMsg::DestroyUser(id) to the DirectoryThread. Sync is implemented for this type so be careful when modifying it
*/
pub struct UserEntry {
    id: Arc<StoredDirectoryId>
}

unsafe impl Send for UserEntry{}

struct StoredDirectoryId {
    directory: Directory,
    id: DirectoryId,
}

impl UserEntry {
    fn new(directory: Directory, id: DirectoryId) -> Self {
        UserEntry{
            id: Arc::new(StoredDirectoryId{
                directory: directory,
                id: id,
            })
        }
    }
}

impl Drop for StoredDirectoryId {
    fn drop(&mut self) {
        println!("Dropping Directory ID");
        // we're about to wipe out the last reference, inform directory thread of its destruction
        self.directory.destroy_user(self.id).unwrap();
    }
}

#[derive(Clone)]
pub struct Directory {
    thread: DirectoryThread,
}

impl Directory {
    pub fn new(thread: DirectoryThread) -> Self {
        Directory{ thread: thread }
    }

    pub fn new_user(&self, user: User) -> Result<UserEntry> {
        let id = try!(req_rep!(self.thread, DirectoryThreadMsg::NewUser => (user)));
        println!("User got id: {:?}", id);
        Ok(UserEntry::new(self.clone(), id))
    }

    pub fn destroy_user(&self, id: DirectoryId) -> Result<()> {
        try!(send!(self.thread, DirectoryThreadMsg::DestroyUser => (id)));
        Ok(())
    }
}


