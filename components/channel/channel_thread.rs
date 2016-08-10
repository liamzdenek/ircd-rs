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
                self.users[i] = Some(user);
                s.send(i);
            },
            ChannelThreadMsg::Part(id) => {
                println!("Got part: {:?}", id);
            },
            ChannelThreadMsg::Privmsg(src, msg) => {

            },
            ChannelThreadMsg::Exit => {
                return true;
            }
        }
        return false;
    }

}
