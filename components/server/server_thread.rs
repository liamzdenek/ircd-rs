use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;

use channel_traits::{Directory};
use net_traits::{Writer,ParsedCommand,ReaderThreadMsg,SRPL};
use server_traits::Config;

#[derive(Debug, Clone)]
enum State {
    Sync,
    Connected,
}

pub struct ServerWorker {
    rx: Receiver<ReaderThreadMsg>,
    writer: Writer,
    directory: Directory,
    config: Config,
    state: State,
}
impl ServerWorker {
    pub fn new(rx: Receiver<ReaderThreadMsg>, writer: Writer, directory: Directory, config: Config) -> Self {
        ServerWorker{
            rx: rx,
            writer: writer,
            directory: directory,
            config: config,
            state: State::Sync,
        }
    }

    pub fn run(&mut self) {
        lprintln!("server worker starting");
    
        self.introduce();
        self.sync();

        loop {
            lselect_timeout!{
                6 * 60 * 1000 => {
                    lprintln!("Connection timed out");
                    return;
                },
                msg = self.rx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            lprintln!("ServerWorker Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        };
    }

    fn introduce(&mut self) {
        self.writer.swrite(SRPL::Pass(self.config.get_server_pass()));

        {
            use net_traits::ProtoOption::*;
            self.writer.swrite(SRPL::ProtoCtl(vec![
                EAUTH(self.config.get_server_name()),
                //SID( ... TODO: this),
                NOQUIT,
                NICKv2,
                SJOIN,
                SJ3,
                CLK,
                NICKIP,
                TKLEXT,
                ESVID,
                MLOCK,
                EXTSWHOIS,
            ]));
        }
        
        self.writer.swrite(SRPL::Server(
            self.config.get_server_name(),
            1, // hops always 1 for self
            self.config.get_server_desc(),
        ));
    }

    fn sync(&mut self) {
        {
            let users = self.directory.get_users().unwrap();
            for user in users.into_iter() {
                let mask = user.get_mask().unwrap();
                self.writer.swrite(SRPL::Nick(
                    mask.nick.clone(),
                    mask.hops.clone(),
                    mask.timestamp.clone(),
                    mask.user.clone(),
                    mask.host.clone(),
                    mask.servername.clone(),
                    "0".into(), // services stamp
                    "".into(), // modes
                    "*".into(), // cloaked host
                    mask.real.clone(),
                ));
            }
        }
        {
            let chans = self.directory.get_channels().unwrap();
            let chans = chans.into_iter().map(|chan| {
                let chan_created_at = 0;
                let chan_name = chan.get_name().unwrap();
                let users = chan.get_users().unwrap().into_iter().map(|user| {
                    user.get_mask().unwrap().nick
                }).collect();
                (chan_name, chan_created_at, users)
            });
            for (chan_name, chan_created_at, users) in chans {
                self.writer.swrite(SRPL::Sjoin(chan_created_at.to_string(), chan_name, users));
            }
        }
        self.writer.swrite(SRPL::EOS);
    }
    
    fn handle_msg(&mut self, msg: ReaderThreadMsg) -> bool {
        return match msg {
            ReaderThreadMsg::Command(cmd) => {
                self.handle_command(cmd)
            },
        };
    }

    
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        match (self.state.clone(), cmd.command.to_uppercase().as_str()) {
            (_, "SMO") => {
                lprintln!("SMO -> {:?}", cmd.trailing.join(" "));
            },
            (_, "PING") => {
                self.writer.swrite(SRPL::Pong(cmd.params.clone().join(" ") + cmd.trailing.clone().join(" ").as_str()));
            },
            (Sync, "EOS") => {
                self.state = State::Connected;
            },
            (Sync, "NICK") => {
                
            },
            _ => {
                lprintln!("I don't know how to handle cmd: {:?}", cmd);
            }
        }
        return false;
    }
}
