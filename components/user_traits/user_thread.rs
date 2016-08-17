use std::sync::mpsc::{channel, Sender};
use super::Result;

pub type UserThread = Sender<UserThreadMsg>;

#[derive(Debug)]
pub struct Mask {
    pub nick: String,
    pub user: String,
    pub host: String,
    pub real: String,
    pub hops: u32,
    pub timestamp: String,
    pub servername: String,
}

impl Mask {
    pub fn new(nick: String, user: String, host: String, real: String, hops: u32, timestamp: String, servername: String) -> Self {
        Mask{
            nick: nick,
            user: user,
            host: host,
            real: real,
            hops: hops,
            timestamp: timestamp,
            servername: servername,
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
    Privmsg(String, String), // Src Mask, Msg
    PrivmsgChan(String, String, String), // Mask, Channel, Msg
    JoinSelf(String),
    PartSelf(String, String), // Channel, Reason
    JoinOther(String, String), // Mask,  Channel
    PartOther(String, String, String), // Mask, Channel, Reason
    GetMask(Sender<Result<Mask>>),
    TransmitNames(String, Vec<String>), // Channel, Names
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

    /*
    pub fn send_command(&self, cmd: ParsedCommand) -> Result<()>{
        try!(send!(self.thread, UserThreadMsg::Command => (cmd)));
        Ok(())
    }
    */

    pub fn privmsg(&self, src: String, msg: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::Privmsg => (src, msg)));
        Ok(())
    }
    
    pub fn privmsg_chan(&self, src: String, chan: String, msg: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::PrivmsgChan => (src, chan, msg)));
        Ok(())
    }

    pub fn inform_self_join(&self, channel: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::JoinSelf => (channel)));
        Ok(())
    }

    pub fn inform_self_part(&self, channel: String, reason: String) -> Result<()> { 
        try!(send!(self.thread, UserThreadMsg::PartSelf => (channel, reason)));
        Ok(())
    }

    pub fn inform_other_join(&self, mask: String, channel: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::JoinOther => (mask, channel)));
        Ok(())
    }

    pub fn inform_other_part(&self, mask: String, channel: String, reason: String) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::PartOther => (mask, channel, reason)));
        Ok(())
    }

    pub fn get_mask(&self) -> Result<Mask> {
        Ok(try!(try!(req_rep!(self.thread, UserThreadMsg::GetMask => ()))))
    }

    pub fn transmit_names(&self, channel: String, names: Vec<String>) -> Result<()> {
        try!(send!(self.thread, UserThreadMsg::TransmitNames => (channel, names)));
        Ok(())
    }
}
