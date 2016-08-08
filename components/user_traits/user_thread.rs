use std::sync::mpsc::{channel, Sender};
use util::*;
use net_traits::ParsedCommand;
use super::Result;

pub type UserThread = Sender<UserThreadMsg>;

#[derive(Debug)]
pub enum UserThreadMsg {
    Command(ParsedCommand),
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
}
