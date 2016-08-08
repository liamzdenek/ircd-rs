use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::collections::HashMap;
use channel_traits::*;
use user_traits::User;
use std::rc::{Rc,Weak};

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

struct DUserEntry {
    thread: User,
    nick: String,
}

pub struct DirectoryWorker {
    rx: Receiver<DirectoryThreadMsg>,
    users: Vec<Option<Rc<DUserEntry>>>,
    users_by_name: HashMap<String, Weak<DUserEntry>>,
}

impl DirectoryWorker {
    fn new(rx: Receiver<DirectoryThreadMsg>) -> Self {
        DirectoryWorker{
            rx: rx,
            users: vec![],
            users_by_name: HashMap::new(),
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
                            println!("DirectoryThread Got error: {:?}", e);
                        }
                    }
                },
            };
        }
    }

    fn handle_msg(&mut self, msg: DirectoryThreadMsg) -> bool{
        println!("Directory Thread got msg: {:?}", msg);
        match msg {
            DirectoryThreadMsg::GetChannels(s) => {

            },
            DirectoryThreadMsg::GetChannelByName(s, name) => {

            },
            DirectoryThreadMsg::GetUserByName(s, name) => {

            },
            DirectoryThreadMsg::NewUser(s, user) => {
                let entry = DUserEntry{
                    thread: user,
                    nick: "".into(),
                };
                let mut i: u64 = 0;
                for (j, v) in self.users.iter().enumerate() {
                    i = j as u64;
                    if v.is_none() {
                        i-=1;
                        break;
                    }
                }
                while self.users.len() <= i as usize {
                    self.users.push(None);
                }
                self.users[i as usize] = Some(Rc::new(entry));
                s.send(i);
                i+=1;
            },
            DirectoryThreadMsg::DestroyUser(id) => {
                self.users[id as usize] = None;
            },
            DirectoryThreadMsg::UpdateNick(s,id,nick) => {
                let nick_in_use = {
                    match self.users.get(id as usize) {
                        Some(&Some(ref user)) => {
                            if user.nick != nick {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                };
                if nick_in_use {
                    s.send(Err(Error::NickCollision));
                    return false;
                }
                match self.users.get(id as usize) {
                    Some(&Some(ref user)) => {
                        self.users_by_name.insert(nick, Rc::downgrade(user));
                    }
                    _ => {}
                }
            },
            DirectoryThreadMsg::Exit => {
                return true;
            },
        }
        return false;
    }
}
