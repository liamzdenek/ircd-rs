use std::sync::mpsc::{Sender};
use super::Result;
use super::ParsedCommand;

pub type ReaderThread = Sender<ReaderThreadMsg>;

pub enum ReaderThreadMsg {
    Command(ParsedCommand)
}

pub type WriterThread = Sender<WriterThreadMsg>;

#[derive(Debug)]
pub enum WriterThreadMsg {
    SendRaw(String),
    Send(RPL),
    SSend(SRPL),
    UpdateNick(String),
}

#[derive(Debug, Clone)]
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

    pub fn swrite(&self, msg: SRPL) -> Result<()> {
        try!(send!(self.thread, WriterThreadMsg::SSend => (msg)));
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
pub enum SRPL {
    Pong(String), //msg
    Pass(String), // password
    Server(String, u32, String), // name, hops, desc
    ProtoCtl(Vec<ProtoOption>),
    Nick(String, u32, String, String, String, String, String, String, String, String), // Nick, Hops, Timestamp, Username, Hostname, Servername, Servicestamp, Modes, CloakedHost, Realname)
    Sjoin(String, String, Vec<String>), // Timestamp, Channel, Vec<Nick with modes>
    EOS,
}

#[derive(Debug)]
pub enum ProtoOption {
    EAUTH(String), // server name
    SID(String), // server id
    NOQUIT,
    NICKv2,
    SJOIN,
    SJ3,
    CLK,
    NICKIP,
    TKLEXT,
    TKLEXT2,
    ESVID,
    MLOCK,
    EXTSWHOIS,
}

impl ProtoOption {
    pub fn raw(&self) -> String {
        use ProtoOption::*;
        match self {
            &EAUTH(ref server_name) => format!("EAUTH={}", server_name),
            &SID(ref server_id) => format!("SID={}",server_id),
            &NOQUIT => "NOQUIT".into(),
            &NICKv2 => "NICKv2".into(),
            &SJOIN => "SJOIN".into(),
            &SJ3 => "SJ3".into(), 
            &CLK => "CLK".into(),
            &NICKIP => "NICKIP".into(),
            &TKLEXT => "TKLEXT".into(),
            &TKLEXT2 => "TKLEXT2".into(),
            &ESVID => "ESVID".into(),
            &MLOCK => "MLOCK".into(),
            &EXTSWHOIS => "EXTSWHOIS".into(),
        }.into()
    }
}

impl SRPL {
    pub fn raw(&self, data: &mut WriterData) -> String {
        let servername = &data.server_name;
        match self {
            &SRPL::Pass(ref pass) => format!("PASS :{pass}",
                pass=pass
            ),
            &SRPL::Server(ref name, ref hops, ref desc) => format!("SERVER {name} {hops} :{desc}",
                name=name,
                hops=hops,
                desc=desc,
            ),
            &SRPL::Pong(ref msg) => format!("PONG :{msg}",
                msg = msg,
            ),
            &SRPL::ProtoCtl(ref opts) => {
                let str = opts.iter().map(|opt| opt.raw()).collect::<Vec<_>>().join(" ");
                format!("PROTOCTL {opts}", opts = str)
            },
            &SRPL::EOS => "EOS".into(),
            &SRPL::Nick(ref nick, ref hops, ref timestamp, ref username, ref hostname, ref servername, ref servicesstamp, ref modes, ref cloakedhost, ref realname) => {
                let hops = hops+1;
                format!("NICK {nick} {hops} {timestamp} {username} {hostname} {servername} {servicesstamp} {modes} {cloakedhost} :{realname}",
                    nick=nick,
                    hops=hops,
                    timestamp=timestamp,
                    username=username,
                    hostname=hostname,
                    servername=servername,
                    servicesstamp=servicesstamp,
                    modes=modes,
                    cloakedhost=cloakedhost,
                    realname=realname,
                )
            }
            &SRPL::Sjoin(ref timestamp, ref channel, ref users) => format!(":{sname} SJOIN {timestamp} {channel} :{users}",
                sname=servername,
                timestamp=timestamp,
                channel=channel,
                users=users.join(" "),
            ),
        }
    }
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
