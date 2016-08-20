use std::sync::mpsc::{channel, Receiver, Select};
use std::net::TcpStream;
use std::thread;

use user_traits::{Mask};
use channel_traits::{Directory};
use net_traits::{Writer,ParsedCommand,ReaderThreadMsg,SRPL};
use server_traits::Config;
use super::VirtualUser;

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
    users: Vec<VirtualUser>,
}
impl ServerWorker {
    pub fn new(rx: Receiver<ReaderThreadMsg>, writer: Writer, directory: Directory, config: Config) -> Self {
        ServerWorker{
            rx: rx,
            writer: writer,
            directory: directory,
            config: config,
            state: State::Sync,
            users: vec![],
        }
    }

    pub fn run(&mut self) {
        lprintln!("server worker starting");
    
        self.introduce();
        self.sync();

        enum SelectState {
            SelfUser(usize),
            SelfRx,
        };

        loop {
            // this block is unsafe because handles may not be moved after .add has been called
            let state: SelectState = unsafe {
                
                let sel = Select::new();
                
                // self.rx
                let mut self_rx = sel.handle(&self.rx);
                self_rx.add();
    
                // users
                let mut self_users: Vec<_> = self.users.iter().map(|user| {
                    sel.handle(user.poll())
                }).collect();

                for mut user in self_users.iter_mut() {
                    user.add();
                }

                lprintln!("=========================");
                lprintln!("= = WAITING = =");
                lprintln!("=========================");
                let id = sel.wait();
                lprintln!("=========================");
                lprintln!("= = DONE WAITING = =");
                lprintln!("=========================");
                if id == self_rx.id() { SelectState::SelfRx }
                else {
                    let user = self_users.iter().enumerate().find(|&(ref i, ref user)| id == user.id());
                    if user.is_some() {
                        SelectState::SelfUser(user.unwrap().0)
                    } else {
                        unreachable!{}
                    }
                }
            };

            match state {
                SelectState::SelfRx => {
                    match self.rx.recv() {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            lprintln!("ServerWorker Got error: {:?}", e);
                            return;
                        }
                    };
                },
                SelectState::SelfUser(i) => {
                    unimplemented!{};
                    match self.users[i].poll().recv() {
                        Ok(msg) => {
                            println!("GOT MSG FOR USER: {:?}", msg);
                            unimplemented!{};
                        },
                        Err(e) => {
                            unimplemented!{};
                        },
                    }
                }
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
            (State::Sync, "EOS") => {
                self.state = State::Connected;
            },
            (_, "NICK") => {
                lprintln!("GOT VIRTUAL USER");
                let mask = Mask::new(
                    cmd.params[0].clone(), // Nick
                    cmd.params[3].clone(), // User
                    cmd.params[4].clone(), // host
                    cmd.trailing.join(".clone() "), // real
                    cmd.params[1].parse().unwrap(), // hops
                    cmd.params[2].clone(), // timestamp
                    cmd.params[5].clone(), // servername
                );
                let vu = VirtualUser::new(self.directory.clone(), self.config.clone(), mask);
                self.users.push(vu);
            },
            (_, "SJOIN") => {
                let timestamp = cmd.params[0].clone();
                let channel = cmd.params[1].clone();
                let nicks = cmd.trailing.clone();
                for nick in nicks.into_iter() {
                    let (nick, modes) = parse_nick(nick);
                    
                    let maybe_user = self.users.iter_mut().find(|user| user.get_mask().nick == nick);
                    
                    if let Some(user) = maybe_user {
                        user.join(channel.clone());
                    }
                }
            },
            _ => {
                lprintln!("I don't know how to handle cmd: {:?}", cmd);
            }
        }
        return false;
    }
}

fn parse_nick(nick: String) -> (String, Vec<char>) {
    let mut chars = nick.chars();
    let mut is_flags = true;
    let mut outnick = String::new();
    let mut modes = vec![];
    while let Some(next) = chars.next() {
        match (is_flags, next) {
            (true, '@') => {
                modes.push('o');
            },
            (_, c) => {
                outnick.push(c);
            },
        }
    };
    (outnick, modes)
}
