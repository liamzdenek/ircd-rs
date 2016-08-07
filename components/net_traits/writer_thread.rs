use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type WriterThread = Sender<WriterThreadMsg>;

#[derive(Debug)]
pub enum WriterThreadMsg {
    SendRaw(String),
    Send(RPL),
    UpdateNick(String),
}

pub struct Writer {
    thread: WriterThread,
}

impl Writer {
    pub fn new(thread: WriterThread) -> Self {
        Writer{ thread: thread }
    }

    pub fn write_raw(&self, msg: String) -> Result<()> {
        try!(send!(self.thread, WriterThreadMsg::SendRaw => (msg)));
        Ok(())
    }

    pub fn write(&self, msg: RPL) -> Result<()> {
        try!(send!(self.thread, WriterThreadMsg::Send => (msg)));
        Ok(())
    }

    pub fn update_nick(&self, nick: String) -> Result<()> {
        try!(send!(self.thread, WriterThreadMsg::UpdateNick => (nick)));
        Ok(())
    }
}

#[derive(Default,Debug)]
pub struct WriterData {
    pub nick: String,
}

#[derive(Debug)]
pub enum RPL {
    Welcome{msg: String},
    YourHost,
    Mode{mode: char, enabled: bool},
    // MOTD
    MotdStart,
    Motd(String),
    MotdEnd,
}

impl RPL {
    pub fn raw(&self, data: &WriterData) -> String {
        //TODO: Proper config
        let servername = "test.localhost";
        match self {
            &RPL::Welcome{ref msg} => format!(":{sname} 001 {nick} :{msg}",
                sname = servername,
                nick = data.nick,
                msg = msg,
            ),
            &RPL::YourHost => format!(":{sname} 002 {nick} :Your host is {sname}",
                sname = servername,
                nick = data.nick,
            ),
            &RPL::Mode{mode, enabled} => format!(":{nick} MODE {nick} :{sym}{mode}",
                nick = data.nick,
                sym = if enabled { "+" } else { "-" },
                mode = mode,
            ),
            &RPL::MotdStart => format!(":{sname} 375 :- {sname} Message of the Day -",
                sname = servername,
            ),
            &RPL::Motd(ref msg) => format!(":{sname} 372 :- {msg}",
                sname = servername,
                msg = msg,
            ),
            &RPL::MotdEnd => format!(":{sname} 376 :End of /MOTD command.", sname = servername),
        }
    }
}
