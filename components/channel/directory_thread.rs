use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::collections::HashMap;
use channel_traits::*;
use user_traits::User;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use super::ChannelThreadFactory;

pub trait DirectoryThreadFactory {
    fn new() -> Self;
}

impl DirectoryThreadFactory for DirectoryThread {
    fn new() -> DirectoryThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("DirectoryThread".to_string()).spawn(move || {
            DirectoryWorker::new(rx).run();
        });
        tx
    }
}

#[derive(Debug)]
struct DUserEntry {
    thread: User,
    nick: String,
}

#[derive(Debug)]
struct DChannelEntry {
    thread: Channel,
}

pub struct DirectoryWorker {
    rx: Receiver<DirectoryThreadMsg>,
    users: Vec<Option<Rc<RefCell<DUserEntry>>>>,
    // todo, replace Rc<_> with Weak<_>, this could lead to potential memleaks otherwise
    // The DestroyUser handler should be very carefully modified as a consequence of this decision
    users_by_nick: HashMap<String, Rc<RefCell<DUserEntry>>>,
    channels_by_name: HashMap<String, DChannelEntry>,
}

impl DirectoryWorker {
    fn new(rx: Receiver<DirectoryThreadMsg>) -> Self {
        DirectoryWorker{
            rx: rx,
            users: vec![],
            users_by_nick: HashMap::new(),
            channels_by_name: HashMap::new(),
        }
    }

    fn run(&mut self) {
        loop {
            lselect!{
                msg = self.rx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            lprintln!("DirectoryThread Got error: {:?}", e);
                        }
                    }
                },
            };
        }
    }

    fn handle_msg(&mut self, msg: DirectoryThreadMsg) -> bool{
        lprintln!("Directory Thread got msg: {:?}", msg);
        match msg {
            DirectoryThreadMsg::GetChannels(s) => {
                s.send(
                    self.channels_by_name.values().map(|stored_chan| {
                        stored_chan.thread.clone()
                    }).collect()
                );
            },
            DirectoryThreadMsg::GetChannelByName(s, name, nick) => {
                let has_new = match self.channels_by_name.get(&name) {
                    Some(ref channel) => {
                        s.send(channel.thread.clone());
                        None
                    },
                    None => {
                        let channel = Channel::new(ChannelThreadFactory::new(name.clone(), nick));
                        s.send(channel.clone());
                        Some(channel)
                    }
                };
                if let Some(channel) = has_new {
                    self.channels_by_name.insert(name, DChannelEntry{ thread: channel.clone() });
                }
            },
            DirectoryThreadMsg::GetUsers(s) => {
                s.send(self.users.clone().into_iter().filter_map(|user|
                    match user {
                        Some(user) => Some(user.borrow().thread.clone()),
                        None => None,
                    }
                ).collect());
            },
            DirectoryThreadMsg::GetUserByNick(s, nick) => {
                s.send(
                    match self.users_by_nick.get(&nick) {
                        Some(user) => {
                            Ok(user.borrow().thread.clone())
                        }
                        None => Err(Error::NickNotFound)
                    }
                );
            },
            DirectoryThreadMsg::NewUser(s, user) => {
                let entry = DUserEntry{
                    thread: user,
                    nick: "".into(),
                };
                let mut i: u64 = 0;
                for (j, v) in self.users.iter().enumerate() {
                    if v.is_none() {
                        i = j as u64;
                        break;
                    } else {
                        i = j as u64 +1;
                    }
                }
                while self.users.len() <= i as usize {
                    self.users.push(None);
                }
                self.users[i as usize] = Some(Rc::new(RefCell::new(entry)));
                s.send(i);
            },
            DirectoryThreadMsg::DestroyUser(id) => {
                let mut nick = None;
                match self.users.get(id as usize) {
                    Some(&Some(ref user)) => {
                        nick = Some(user.borrow().nick.clone());
                    }
                    _ => {}
                }
                lprintln!("=========================");
                lprintln!("unregistering user: {:?}", nick);
                lprintln!("=========================");
                match nick {
                    Some(nick) => {
                        self.users_by_nick.remove(&nick);
                    }
                    None => {}
                }
                self.users[id as usize] = None;
            },
            DirectoryThreadMsg::UpdateNick(s,id,nick) => {
                lprintln!("Updating nick: {:?} |||||| {:?} |||||| {:?}", nick, self.users, self.users_by_nick);
                let nick_in_use = {
                    match self.users_by_nick.get(&nick) {
                        Some(user) => {
                            lprintln!("UpdateNick Got user: {:?}", user);
                            true
                            /*
                            match user.upgrade() {
                                Some(user) => {
                                    lprintln!("Upgraded");
                                    true
                                }
                                None => {
                                    lprintln!("Not Upgraded");
                                    false
                                }
                            }*/
                        }
                        _ => false,
                    }
                };
                if nick_in_use {
                    lprintln!("Nick in use");
                    s.send(Err(Error::NickCollision));
                    return false;
                }
                match self.users.get_mut(id as usize) {
                    Some(&mut Some(ref user)) => {
                        {
                            let mut tuser = user.borrow_mut();
                            tuser.nick = nick.clone().into();
                        }
                        self.users_by_nick.insert(nick.clone(), user.clone());//Rc::downgrade(user));
                        //lprintln!("ATTEMT IMMEDIATE UPGRADE: {:?}", self.users_by_nick.get(&nick).unwrap().upgrade());
                    }
                    _ => {}
                }
                s.send(Ok(()));
            },
            DirectoryThreadMsg::Exit => {
                return true;
            },
        }
        return false;
    }
}
