use std::sync::mpsc::{channel, Receiver};
use std::thread;

use net_traits::{Writer, ParsedCommand, RPL};
use user_traits::*;
use channel_traits::{Directory, DirectoryEntry, Channel, ChannelEntry};
use channel_traits::error::Error as channel_traits_error;

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

#[derive(Debug)]
struct StoredChannel {
    name: String,
    thread: ChannelEntry,
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

    fn gen_mask(&self) -> Mask {
        Mask::new(self.nick.clone(), self.user_name.clone(), "TODO".into(), self.real_name.clone())
    }
}

pub struct UserWorker {
    rx: Receiver<UserThreadMsg>,
    directory: Directory,
    directory_entry: DirectoryEntry,
    channels: Vec<StoredChannel>,
    writer: Writer,
    state: State,
    modes: Vec<char>,
}

impl UserWorker {
    fn new(rx: Receiver<UserThreadMsg>, writer: Writer, directory: Directory, directory_entry: DirectoryEntry) -> Self {
        UserWorker{
            rx: rx,
            directory_entry: directory_entry,
            directory: directory,
            writer: writer,
            state: State::NewConnection(None),
            channels: vec![],
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
            UserThreadMsg::Privmsg(src, msg) => {
                //println!("Received Privmsg -- <{}> {}", src, msg);
                self.writer.write(RPL::Privmsg(src, msg));
                false
            },
            UserThreadMsg::Exit => {
                true
            }
        }
    }
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        //TODO: handle htis better so that self.state is not cloned
        match (self.state.clone(), cmd.command.to_uppercase().as_ref()) {
            // TODO: add PASSWD support
            (State::NewConnection(maybe_data), "NICK") |
            (State::NewConnection(maybe_data), "USER") => {
                let mut data = maybe_data.unwrap_or(Default::default());
                data.apply(cmd);
                println!("checking is ready {:?}", data);
                self.state = if data.is_ready() {
                    println!("== Connected");
                    self.writer.update_nick(data.nick.clone());
                    let has_collisions = self.directory_entry.update_nick(data.nick.clone());
                    println!("GOT BACK: {:?}", has_collisions);
                    match has_collisions {
                        Ok(_) => {
                            println!("Nick has no collisions, good to continue");
                        }
                        Err(channel_traits_error::NickCollision) => {
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
            (State::Connected{data}, "PRIVMSG") => {
                match self.directory.get_user_by_nick(cmd.params[0].clone()) {
                    Ok(user) => {
                        let string = cmd.params.split_at(1).1.join(" ") + cmd.trailing.join(" ").as_ref();
                        
                        user.privmsg(data.gen_mask().for_privmsg(), string);
                    },
                    Err(channel_traits_error::NickNotFound) => {
                        self.writer.write(RPL::NickNotFound(cmd.params[0].clone()));
                    },
                    Err(_) => {

                    },
                };
            },
            (State::Connected{data}, "JOIN") => {
                let name = cmd.params[0].clone();
                if self.is_in_channel(&name) {
                    println!("Already in channel, doing nothing");
                    return false;
                }
                match self.directory.get_channel_by_name(name.clone(), data.nick.clone()) {
                    Ok(channel) => {
                        println!("Got channel: {:?}", channel);
                        match self.directory.get_user_by_nick(data.nick.clone()) {
                            Ok(user) => {
                                match channel.join(user) {
                                    Ok(entry) => {
                                        println!("Got entry: {:?}", entry);
                                        self.channels.push(StoredChannel{
                                            name: name.clone(),
                                            thread: entry,
                                        });
                                    },
                                    Err(e) => {
                                        println!("Error during join process: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error getting self to join channel: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error joining channel: {:?}", e);
                    }
                };
            },
            (_, "QUIT") => {
                return true;
            },
            (_,_) => {
                println!("I don't know how to handle CMD: {:?} at with STATE: {:?}", cmd, self.state);
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

    fn is_in_channel(&mut self, name: &String) -> bool{
        self.channels.iter().find(|c| c.name == *name).is_some()
    }
}
