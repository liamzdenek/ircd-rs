use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::collections::HashMap;
use channel_traits::*;
use user_traits::User;
use std::rc::{Rc,Weak};
use std::cell::RefCell;

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

pub struct DirectoryWorker {
    rx: Receiver<DirectoryThreadMsg>,
    users: Vec<Option<Rc<RefCell<DUserEntry>>>>,
    // todo, replace Rc<_> with Weak<_>, this could lead to potential memleaks otherwise
    users_by_nick: HashMap<String, Rc<RefCell<DUserEntry>>>,
}

impl DirectoryWorker {
    fn new(rx: Receiver<DirectoryThreadMsg>) -> Self {
        DirectoryWorker{
            rx: rx,
            users: vec![],
            users_by_nick: HashMap::new(),
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
                self.users[i as usize] = Some(Rc::new(RefCell::new(entry)));
                s.send(i);
                i+=1;
            },
            DirectoryThreadMsg::DestroyUser(id) => {
                self.users[id as usize] = None;
            },
            DirectoryThreadMsg::UpdateNick(s,id,nick) => {
                println!("Updating nick: {:?} |||||| {:?} |||||| {:?}", nick, self.users, self.users_by_nick);
                let nick_in_use = {
                    match self.users_by_nick.get(&nick) {
                        Some(user) => {
                            println!("UpdateNick Got user: {:?}", user);
                            true
                            /*
                            match user.upgrade() {
                                Some(user) => {
                                    println!("Upgraded");
                                    true
                                }
                                None => {
                                    println!("Not Upgraded");
                                    false
                                }
                            }*/
                        }
                        _ => false,
                    }
                };
                if nick_in_use {
                    println!("Nick in use");
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
                        //println!("ATTEMT IMMEDIATE UPGRADE: {:?}", self.users_by_nick.get(&nick).unwrap().upgrade());
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
