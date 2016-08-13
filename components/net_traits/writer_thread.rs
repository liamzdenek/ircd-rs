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
    pub cur_chan: String,
    pub server_name: String,
}

#[derive(Debug)]
pub enum RPL {
    Welcome{msg: String},
    YourHost,
    // Mode
    ModeSelf{mode: char, enabled: bool},
    Mode{target: String, mode: char, enabled: bool},
    // MOTD
    MotdStart,
    Motd(String),
    MotdEnd,
    // NICK
    NickInUse,
    NickNotFound(String),
    //NICK,
    // ping
    Pong(String),
    //CHAT
    Privmsg(String, String), // Mask, Message
    PrivmsgChan(String, String, String), // Mask, Chan, Message
    //
    Join(String, String), // Mask, ChannelName
    Part(String, String, String), // Mask, ChannelName, Reason

    WhoReply(String),
    WhoSpcRpl(String, String), // Mask, Modes
    EndOfWho,

    NameReply(String, Vec<String>), // ChannelName, Names
    EndOfNames(String), // ChannelName
}

impl RPL {
    pub fn raw(&self, data: &mut WriterData) -> String {
        //TODO: Proper config
        let servername = &data.server_name;
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
            &RPL::ModeSelf{mode, enabled} => format!(":{nick} MODE {nick} :{sym}{mode}",
                nick = data.nick,
                sym = if enabled { "+" } else { "-" },
                mode = mode,
            ),
            &RPL::Mode{ref target, mode, enabled} => format!(":{sname} MODE {target} {sym}{mode}",
                target = target,
                sname = servername,
                sym = if enabled { "+" } else { "-" },
                mode = mode
            ),
            &RPL::MotdStart => format!(":{sname} 375 :- {sname} Message of the Day -",
                sname = servername,
            ),
            &RPL::Motd(ref msg) => format!(":{sname} 372 :- {msg}",
                sname = servername,
                msg = msg,
            ),
            &RPL::MotdEnd => format!(":{sname} 376 :End of /MOTD command.", sname = servername),
            &RPL::NickInUse => format!(":{sname} 433 * {nick} :Nickname is already in use.",
                sname = servername,
                nick = data.nick,
            ),
            &RPL::NickNotFound(ref target) => format!(":{sname} 401 {nick} {target} :No such nick/channel",
                sname = servername,
                nick = data.nick,
                target = target,
            ),
            &RPL::Privmsg(ref mask, ref msg) => format!(":{mask} PRIVMSG {nick} :{msg}",
                mask = mask,
                nick = data.nick,
                msg = msg,
            ),
            &RPL::PrivmsgChan(ref mask, ref chan, ref msg) => format!(":{mask} PRIVMSG {chan} :{msg}",
                mask = mask,
                chan = chan,
                msg = msg,
            ),
            &RPL::Pong(ref msg) => format!(":{sname} PONG {sname} :{msg}",
                sname = servername,
                msg = msg,
            ),
            &RPL::Join(ref mask, ref chan) => format!(":{mask} JOIN {chan}",
                mask=mask,
                chan=chan
            ),
            &RPL::Part(ref mask, ref chan, ref reason) => format!(":{mask} PART {chan} :\"{reason}\"",
                mask=mask,
                chan=chan,
                reason=reason,
            ),
            &RPL::WhoReply(ref chan) => {
                data.cur_chan = chan.clone().into();
                format!(":{sname} 352 {chan} %ctnf,152",
                    sname=servername,
                    chan=chan,
                )
            }
            &RPL::WhoSpcRpl(ref mask, ref modes) => format!(":{sname} 354 {mask} 152 {chan} {mask} {modes}",
                sname=servername,
                chan=data.cur_chan,
                mask=mask,
                modes=modes,
            ),
            &RPL::EndOfWho => format!(":{sname} 315 {nick} {chan} :End of /WHO list.",
                sname=servername,
                nick=data.nick,
                chan=data.cur_chan,
            ),
            &RPL::NameReply(ref channel, ref names) => format!(":{sname} 353 {nick} @ {channel} :{names}",
                sname=servername,
                nick=data.nick,
                channel=channel,
                names=names.join(" "),
            ),
            &RPL::EndOfNames(ref channel) => format!(":{sname} 366 {nick} {channel} :End of /NAMES list",
                sname=servername,
                nick=data.nick,
                channel=channel,
            ),
        }
    }
}
