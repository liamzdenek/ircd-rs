use std::sync::mpsc::{channel, Sender};
use util::*;
use net_traits::ParsedCommand;
use super::Result;

pub type UserThread = Sender<UserThreadMsg>;

#[derive(Debug)]
pub struct Mask {
    pub nick: String,
    pub user: String,
    pub host: String,
    pub real: String,
}

impl Mask {
    pub fn new(nick: String, user: String, host: String, real: String) -> Self {
        Mask{
            nick: nick,
            user: user,
            host: host,
            real: real,
        }
    }
    pub fn full(&self) -> String {
        let mut ret = String::new();
        ret.push_str(self.nick.as_ref());
        ret.push_str("!");
        ret.push_str(self.user.as_ref());
        ret.push_str("@");
        ret.push_str(self.host.as_ref());
        ret.push_str(" * ");
        ret.push_str(self.real.as_ref());
        ret
    }
    pub fn for_privmsg(&self) -> String {
        let mut ret = String::new();
        ret.push_str(self.nick.as_ref());
        ret.push_str("!");
        ret.push_str(self.user.as_ref());
        ret.push_str("@");
        ret.push_str(self.host.as_ref());
        ret
    }
}

#[derive(Debug)]
pub enum UserThreadMsg {
    Command(ParsedCommand),
    Privmsg(Mask, String),
    Exit,
}

#[derive(Debug, Clone)]
pub struct User {
    thread: UserThread,
}

impl User {
    pub fn new(thread: UserThread) -> Self {
        User{
            thread: thread,
        }
    }

    pub fn send_command(&self, cmd: ParsedCommand) -> Result<()>{
        try!(send!(self.thread, UserThreadMsg::Command => (cmd)));
        Ok(())
    }

    pub fn privmsg(&self, src: Mask, msg: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::Privmsg => (src, msg)));
        Ok(())
    }
}
