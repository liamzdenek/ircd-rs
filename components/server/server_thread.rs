use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;

use channel_traits::{Directory};
use net_traits::{Writer,ParsedCommand,ReaderThreadMsg,SRPL};
use server_traits::Config;

enum State {
    Sync,
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
                TKLEXT2,
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
        
        self.writer.swrite(SRPL::EOS);
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
    
    fn handle_msg(&mut self, msg: ReaderThreadMsg) -> bool {
        return match msg {
            ReaderThreadMsg::Command(cmd) => {
                self.handle_command(cmd)
            },
        };
    }

    
    fn handle_command(&mut self, mut cmd: ParsedCommand) -> bool{
        lprintln!("Got command: {:?}", cmd);
        match (&self.state, cmd.command.to_uppercase().as_str()) {
            
            (_, "PING") => {
                self.writer.swrite(SRPL::Pong(cmd.params.clone().join(" ")));
            },
            _ => {
                lprintln!("I don't know how to handle cmd: {:?}", cmd);
            }
        }
        return false;
    }
}
