use std::sync::mpsc::{channel, Receiver};
use std::thread;
use net_traits::{Writer, ParsedCommand};

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

#[derive(Debug)]
enum State {
    NewConnection,
    AwaitingUSER{nick: String},
    Connected{data: UserData},
}

#[derive(Debug)]
struct UserData {
    nick: String,
    user_name: String,
    real_name: String,
}

pub struct UserWorker {
    rx: Receiver<UserThreadMsg>,
    writer: Writer,
    state: State,
}

impl UserWorker {
    fn new(rx: Receiver<UserThreadMsg>, writer: Writer) -> Self {
        UserWorker{ rx: rx, writer: writer, state: State::NewConnection }
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
        println!("got msg: {:?}", msg);
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
        self.state = match (&self.state, cmd.command.as_ref()) {
            (&State::NewConnection, "NICK") => {
                println!("got NICK command");
                State::AwaitingUSER{ nick: cmd.params.drain(..).next().unwrap() }
            },
            (&State::AwaitingUSER{ref nick}, "USER") => {
                println!("got USER command");
                let data = UserData {
                    nick: nick.clone().to_owned(),
                    user_name: cmd.params[0].clone(),
                    real_name: cmd.trailing.clone().join(" "),
                };

                println!("User data: {:?}", data);
                self.writer.write_raw("hello world".into());
                
                State::Connected{data: data}
            },
            (_,_) => {
                println!("I don't know how to handle CMD: {:?} at with STATE: {:?}", cmd.command, self.state);
                return true;
            }
        };
        return false;
    }
}
