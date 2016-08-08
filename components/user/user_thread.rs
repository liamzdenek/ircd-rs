use std::sync::mpsc::{channel, Receiver};
use std::thread;

use net_traits::{Writer, ParsedCommand, RPL};
use user_traits::{User, UserThread, UserThreadMsg};
use channel_traits::{Directory, UserEntry};
use channel_traits::error as channel_traits_error;

pub trait UserThreadFactory {
    fn new(w: Writer, directory: Directory) -> Self;
}

impl UserThreadFactory for UserThread {
    fn new(w: Writer, directory: Directory) -> UserThread {
        let (tx,rx) = channel();
        let user = User::new(tx.clone());
        let entry = directory.new_user(user).unwrap();
        thread::Builder::new().name("UserThread".to_string()).spawn(move || {
            UserWorker::new(rx, w, directory, entry).run();
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
    directory: Directory,
    user_entry: UserEntry,
    writer: Writer,
    state: State,
    modes: Vec<char>,
}

impl UserWorker {
    fn new(rx: Receiver<UserThreadMsg>, writer: Writer, directory: Directory, user_entry: UserEntry) -> Self {
        UserWorker{
            rx: rx,
            user_entry: user_entry,
            directory: directory,
            writer: writer,
            state: State::NewConnection(None),
            modes: vec![]
        }
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
                println!("checking is ready {:?}", data);
                self.state = if data.is_ready() {
                    println!("== Connected");
                    self.writer.update_nick(data.nick.clone());
                    let has_collisions = self.user_entry.update_nick(data.nick.clone());
                    println!("GOT BACK: {:?}", has_collisions);
                    match has_collisions {
                        Ok(_) => {
                            println!("Nick has no collisions, good to continue");
                        }
                        Err(channel_traits_error::Error::NickCollision) => {
                            println!("Nick has collisions, cannot continue");
                            self.writer.write(RPL::NickInUse);
                            self.state = State::NewConnection(Some(data));
                            return false;
                        }
                        Err(e) => {
                            println!("Internal error determining if nick has collisions: {:?}", e);
                            self.state = State::NewConnection(Some(data));
                            return false;
                        }
                    }
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
            /*(State::Connected{data}, "JOIN") => {
                
            },*/
            (_, "QUIT") => {
                return true;
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
