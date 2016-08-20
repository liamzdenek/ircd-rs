use std::sync::mpsc::{channel, Receiver, Select, Handle};
use user_traits::{User,UserThreadMsg,Mask};
use channel_traits::{Directory, DirectoryEntry, ChannelEntry};
use server_traits::Config;

#[derive(Debug)]
struct StoredChannel {
    name: String,
    thread: ChannelEntry,
}

pub struct VirtualUser {
    rx: Receiver<UserThreadMsg>,
    directory: Directory,
    config: Config,
    directory_entry: DirectoryEntry,
    channels: Vec<StoredChannel>,
    mask: Mask,
}

impl VirtualUser {
    pub fn new(directory: Directory, config: Config, mask: Mask) -> Self {
        unsafe{
            let (tx,rx) = channel(); // tx MUST scope out during this fn in order to ensure proper cleanup
            let user = User::new(tx);
            let entry = directory.new_user(user).unwrap();
            entry.update_nick(mask.nick.clone()).unwrap();
            VirtualUser{
                rx:rx,
                config: config,
                directory: directory,
                directory_entry: entry,
                mask: mask,
                channels: vec![],
            }
        }
    }

    pub fn get_mask(&self) -> &Mask {
        &self.mask
    }

    pub fn poll(&self) -> &Receiver<UserThreadMsg> {
        &self.rx
    }
    
    pub fn join(&mut self, chan: String) {
        match self.directory.get_channel_by_name(chan.clone(), self.mask.nick.clone()) {
            Ok(channel) => {
                match self.directory.get_user_by_nick(self.mask.nick.clone()) {
                    Ok(user) => {
                        lprintln!("Attempting join");
                        match channel.join(user) {
                            Ok(entry) => {
                                lprintln!("VIRTUAL USER ADDING: {:?}", entry);
                                entry.update_mask(self.mask.nick.clone());
                                self.channels.push(StoredChannel{
                                    name: chan.clone(),
                                    thread: entry,
                                });
                            },
                            _ => {
                                unimplemented!{};
                            },
                        }
                    },
                    _ => {
                        unimplemented!{};
                    },
                }
            },
            _ => {
                unimplemented!{};
            },
        }
    }
}
