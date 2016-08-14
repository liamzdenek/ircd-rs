use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;

use channel_traits::{Directory};
use net_traits::{Writer,ParsedCommand,ReaderThreadMsg,RPL};
use server_traits::Config;

pub struct ServerWorker {
    rx: Receiver<ReaderThreadMsg>,
    writer: Writer,
    directory: Directory,
    config: Config,
}
impl ServerWorker {
    pub fn new(rx: Receiver<ReaderThreadMsg>, writer: Writer, directory: Directory, config: Config) -> Self {
        ServerWorker{
            rx: rx,
            writer: writer,
            directory: directory,
            config: config,
        }
    }

    pub fn run(&mut self) {
        println!("server worker starting");
    
        self.writer.write(RPL::Pass(self.config.get_server_pass()));
        self.writer.write(RPL::Server(
            self.config.get_server_name(),
            1, // hops always 1 for self
            self.config.get_server_desc(),
        ));

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
    
    fn handle_msg(&mut self, msg: ReaderThreadMsg) -> bool {
        return match msg {
            ReaderThreadMsg::Command(cmd) => {
                self.handle_command(cmd)
            },
        };
    }

    
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        println!("Got command: {:?}", cmd);
        match cmd.command.to_uppercase().as_str() {
            "PING" => {
                self.writer.write(RPL::Pong(cmd.params.clone().join(" ")));
            },
            _ => {
                println!("I don't know how to handle cmd: {:?}", cmd);
            }
        }
        return false;
    }
}
