use std::sync::mpsc::{channel, Receiver};
use std::thread;
use channel_traits::*;
use user_traits::User;

pub trait ChannelThreadFactory {
    fn new(name: String, nick: String) -> Self;
}

impl ChannelThreadFactory for ChannelThread {
    fn new(name: String, nick: String) -> ChannelThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("ChannelThread".to_string()).spawn(move || {
            ChannelWorker::new(rx, name, nick).run();
        });
        tx
    }
}

pub struct ChannelWorker {
    rx: Receiver<ChannelThreadMsg>,
    name: String,
    nick: String,
    users: Vec<Option<User>>,
}

impl ChannelWorker {
    fn new(rx: Receiver<ChannelThreadMsg>, name: String, nick: String) -> Self {
        ChannelWorker{
            rx: rx,
            name: name,
            nick: nick,
            users: vec![],
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
    
    fn handle_msg(&mut self, msg: ChannelThreadMsg) -> bool {
        match msg {
            ChannelThreadMsg::Join(s, user) => {
                println!("Got join: {:?}", user);
                let mut i = 0;
                for (j,v) in self.users.iter().enumerate() {
                    if v.is_none() {
                        i = j;
                        break;
                    } else {
                        i = j + 1;
                    }
                }
                while self.users.len() <= i {
                    self.users.push(None);
                }
                s.send(i); // must come before introduce/welcome otherwise may cause deadlock
                self.introduce(&user);
                self.welcome(&user);
                self.users[i] = Some(user);
            },
            ChannelThreadMsg::Part(id, reason) => {
                println!("Got part: {:?}", id);
                let found = match self.users.get(id) {
                    Some(&Some(ref user)) => {
                        user.inform_self_part(self.name.clone(), reason.unwrap_or("No Reason Provided".into()));
                        true
                    }
                    _ => false
                };
                if found {
                    self.users[id] = None;
                };
            },
            ChannelThreadMsg::Who(id) => {
                let users = self.users.clone().into_iter().enumerate().filter_map(|(tid, user)| {
                    user
                });
                match self.users.get(id) {
                    Some(&Some(ref user)) => {
                        let mut names = vec![];
                        for member in users {
                            if let Ok(mask) = member.get_mask() {
                                names.push(mask.nick.to_owned())
                            }
                        }
                        user.transmit_names(self.name.clone(), names);
                    },
                    _ => {}
                }
            },
            ChannelThreadMsg::Privmsg(id, mask, msg) => {
                println!("[{chan}] <{mask}> {msg}", chan=self.name, mask=mask, msg=msg);
                for (tid, user) in self.users.iter().enumerate() {
                    if id == tid {
                        continue;
                    }
                    match user {
                        &Some(ref user) => {
                            user.privmsg_chan(mask.clone(), self.name.clone(), msg.clone());
                        },
                        _ => {}
                    }
                }
            },
            ChannelThreadMsg::Exit => {
                return true;
            }
        }
        return false;
    }

    fn introduce(&mut self, user: &User) {
        let mask = user.get_mask().unwrap().for_privmsg();
        for tuser in self.users.iter() {
            match tuser {
                &Some(ref tuser) => {tuser.inform_other_join(mask.clone(), self.name.clone());},
                _ => {},
            }
        }
    }

    fn welcome(&mut self, user: &User) {
        user.inform_self_join(self.name.clone());
    }

}
