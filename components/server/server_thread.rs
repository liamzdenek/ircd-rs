use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;

use channel_traits::{Directory};
use user_traits::{UserThreadMsg};
use net_traits::{Writer,ParsedCommand};

pub struct ServerWorker {
    rx: Receiver<UserThreadMsg>,
    writer: Writer,
    directory: Directory,
}
impl ServerWorker {
    pub fn new(rx: Receiver<UserThreadMsg>, writer: Writer, directory: Directory) -> Self {
        ServerWorker{
            rx: rx,
            writer: writer,
            directory: directory,
        }
    }

    pub fn run(&mut self) {
        println!("server worker starting");
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
                            println!("ServerWorker Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        };
    }
    
    fn handle_msg(&mut self, msg: UserThreadMsg) -> bool {
        return match msg {
            UserThreadMsg::Command(cmd) => {
                self.handle_command(cmd)
            },
            UserThreadMsg::JoinOther(..) |
            UserThreadMsg::JoinSelf(..) |
            UserThreadMsg::PartOther(..) |
            UserThreadMsg::PartSelf(..) |
            UserThreadMsg::TransmitNames(..) |
            UserThreadMsg::GetMask(..) |
            UserThreadMsg::Privmsg(..) |
            UserThreadMsg::PrivmsgChan(..) => {
                println!("ServerWorker cannot handle this msg: {:?}", msg);
                false
            },
            UserThreadMsg::Exit => {
                println!("ServerWorker exiting");
                true
            },
        };
    }

    
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        println!("Got command: {:?}", cmd);
        return false;
    }
}
