use std::sync::mpsc::{channel, Receiver};
use std::thread;
use net_traits::{Writer, ParsedCommand, RPL};

use user_traits::{UserThread, UserThreadMsg};

pub trait UserThreadFactory {
    fn new(w: Writer) -> Self;
}

impl UserThreadFactory for UserThread {
    fn new(w: Writer) -> UserThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("UserThread".to_string()).spawn(move || {
            UserWorker::new(rx, w).run();
        });

        tx
    }
}

#[derive(Debug, Clone)]
enum State {
    NewConnection(Option<UserData>),
    Connected{data: UserData},
}

#[derive(Debug, Default, Clone)]
struct UserData {
    nick: String,
    user_name: String,
    real_name: String,
}

impl UserData {
    fn apply(&mut self, mut cmd: ParsedCommand) {
        if cmd.command == "NICK" {
            self.nick = cmd.params.drain(..).next().unwrap();
        } else if cmd.command == "USER" {
            self.user_name = cmd.params[0].clone();
            self.real_name = cmd.trailing.clone().join(" ");
        }
    }

    fn is_ready(&mut self) -> bool {
        return self.nick.len() > 0 && self.user_name.len() > 0 && self.real_name.len() > 0
    }
}

pub struct UserWorker {
    rx: Receiver<UserThreadMsg>,
    writer: Writer,
    state: State,
    modes: Vec<char>,
}

impl UserWorker {
    fn new(rx: Receiver<UserThreadMsg>, writer: Writer) -> Self {
        UserWorker{ rx: rx, writer: writer, state: State::NewConnection(None), modes: vec![] }
    }

    fn run(&mut self) {
        println!("user worker starting");
        loop {
            lselect_timeout!{
                6 * 60 * 1000 => {
                    println!("Connection timed out");
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
                            println!("UserThread Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        }
    }

    fn handle_msg(&mut self, msg: UserThreadMsg) -> bool {
        //println!("got msg: {:?}", msg);
        return match msg {
            UserThreadMsg::Command(cmd) => {
                self.handle_command(cmd)
            },
            UserThreadMsg::Exit => {
                true
            }
        }
    }
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        //TODO: handle htis better so that self.state is not cloned
        match (self.state.clone(), cmd.command.as_ref()) {
            // TODO: add PASSWD support
            (State::NewConnection(maybe_data), "NICK") |
            (State::NewConnection(maybe_data), "USER") => {
                let mut data = maybe_data.unwrap_or(Default::default());
                data.apply(cmd);
                self.state = if data.is_ready() {
                    println!("== Connected");
                    self.writer.update_nick(data.nick.clone());
                    self.introduce(&data);
                    self.welcome(&data);
                    State::Connected{data: data}
                } else {
                    State::NewConnection(Some(data))
                }
            },
            (_, "PING") => {
                self.writer.write(RPL::Pong(cmd.params[0].clone()));
            },
            (_,_) => {
                println!("I don't know how to handle CMD: {:?} at with STATE: {:?}", cmd.command, self.state);
                //return true;
            }
        };
        return false;
    }

    fn introduce(&mut self, data: &UserData) {
        //TODO: broadcast to the other servers information about this user, refer to seven src/s_user.c introduce_client
    }

    fn welcome(&mut self, data: &UserData) {
        // upon first connect send the user this information
        self.writer.write(RPL::Welcome{msg: "Hello, World!".into()});
        self.writer.write(RPL::YourHost);
        self.motd();
        self.set_mode('i');
    }

    fn motd(&mut self) {
        self.writer.write(RPL::MotdStart);
        self.writer.write(RPL::Motd("Hello MOTD".into()));
        self.writer.write(RPL::MotdEnd);
    }

    fn set_mode(&mut self, mode: char) {
        if !self.modes.contains(&mode) {
            self.modes.push(mode);
        }
        self.writer.write(RPL::ModeSelf{mode: mode, enabled: true});
    }
    
    fn remove_mode(&mut self, mode: char) {
        self.modes.retain(|e| (*e) != mode);
        self.writer.write(RPL::ModeSelf{mode: mode, enabled: false});
    }
}
