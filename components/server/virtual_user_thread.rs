use std::sync::mpsc::{channel, Receiver, Select, Handle};
use user_traits::{User,UserThread,UserThreadMsg,Mask};
use channel_traits::{Directory, DirectoryEntry, ChannelEntry};
use server_traits::Config;
use std::thread;
use server_traits::{VirtualUser, VirtualUserThreadMsg};

#[derive(Debug)]
struct StoredChannel {
    name: String,
    thread: ChannelEntry,
}

pub struct VirtualUserChannels {
    pub user_thread: User,
    pub vuser_thread: VirtualUser,
}

pub trait VirtualUserThreadFactory {
    fn new(Directory, Config, Mask) -> VirtualUserChannels;
}

impl VirtualUserThreadFactory for UserThread {
    fn new(directory: Directory, config: Config, mask: Mask) -> VirtualUserChannels {
        let (utx,urx) = channel();
        let (vtx,vrx) = channel();
        let user = User::new(utx.clone());
        let entry = directory.new_user(user.clone()).unwrap();
        thread::Builder::new().name("VirtualUserThread".to_string()).spawn(move || {
            VirtualUserWorker::new(urx, vrx, entry, directory, config, mask).run();
        });
        VirtualUserChannels{
            user_thread: user,
            vuser_thread: VirtualUser::new(vtx.clone()),
        }
    }
}

struct VirtualUserWorker {
    urx: Receiver<UserThreadMsg>,
    vrx: Receiver<VirtualUserThreadMsg>,
    directory: Directory,
    config: Config,
    directory_entry: DirectoryEntry,
    channels: Vec<StoredChannel>,
    mask: Mask,
}

impl VirtualUserWorker {
    pub fn new(urx: Receiver<UserThreadMsg>, vrx: Receiver<VirtualUserThreadMsg>, entry: DirectoryEntry, directory: Directory, config: Config, mask: Mask) -> Self {
        entry.update_nick(mask.nick.clone()).unwrap();
        VirtualUserWorker{
            urx:urx,
            vrx:vrx,
            config: config,
            directory: directory,
            directory_entry: entry,
            mask: mask,
            channels: vec![],
        }
    }

    fn run(&mut self) {
        loop {
            lselect!{
                msg = self.vrx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_vuser_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            lprintln!("VirtualUserThread Got error from server side: {:?}", e);
                            return;
                        }
                    }
                },
                msg = self.urx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_user_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            lprintln!("VirtualUserThread Got error from user side: {:?}", e);
                            return;
                        }
                    }
                },
            };
        }
    }

    pub fn handle_user_msg(&mut self, msg: UserThreadMsg) -> bool{
        lprintln!("GOT USER MSG: {:?}", msg);
        match msg {
            UserThreadMsg::GetMask(s) => {
                s.send(Ok(self.mask.clone()));
            },
            UserThreadMsg::Privmsg(nick, msg) => {
                unimplemented!{};
            },
            UserThreadMsg::PrivmsgChan(nick, chan, msg) => {
                unimplemented!{};
            },
            UserThreadMsg::Exit => {
                return true
            },
            UserThreadMsg::JoinSelf(chan) => {
                // nothing to do, this is when the channel thread announces that you have joined, but sending this event is the remote server's job
            },
            UserThreadMsg::JoinOther(chan, nick) => {
                // nothing to do ^^^
            },
            UserThreadMsg::PartSelf(chan, reason) => {
                // nothing to do ^^^
            },
            UserThreadMsg::PartOther(mask, chan, reason) => {
                // nothing to do ^^^
            },
            UserThreadMsg::TransmitNames(chan, names) => {
                // nothing to do ^^^
            },
        }
        false
    }

    pub fn handle_vuser_msg(&mut self, msg: VirtualUserThreadMsg) -> bool {
        lprintln!("GOT VIRTUAL USER MSG: {:?}", msg);
        match msg {
            VirtualUserThreadMsg::Join(chan) => {
                self.join(chan);
            },
            VirtualUserThreadMsg::Part(chan) => {
                self.part(chan);
            },
            VirtualUserThreadMsg::Exit => {
                return true;
            },
        }
        false
    }

    pub fn part(&mut self, chan: String) {
        let maybe_chan = self.channels.iter().enumerate().find(|&(i, ref schan)| schan.name == chan).map(|(i, ref schan)| i);
        if let Some(i) = maybe_chan {
            lprintln!("Swap removing channel: {:?}", chan);
            let chan = self.channels.swap_remove(i);
        }
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
