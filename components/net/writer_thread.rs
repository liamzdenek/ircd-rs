use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;
use std::io::Write;

use net_traits::*;
use server_traits::Config;

pub trait WriterThreadFactory {
    fn new(TcpStream, Config) -> Self;
}

impl WriterThreadFactory for WriterThread {
    fn new(stream: TcpStream, config: Config) -> WriterThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("WriterThread".to_string()).spawn(move || {
            WriterWorker::new(stream, rx, config).run();
        });
        tx
    }
}

pub struct WriterWorker {
    stream: TcpStream,
    rx: Receiver<WriterThreadMsg>,
    data: WriterData,
    config: Config,
}

impl WriterWorker {
    fn new(stream: TcpStream, rx: Receiver<WriterThreadMsg>, config: Config) -> Self {
        WriterWorker{
            stream: stream,
            rx: rx,
            data: Default::default(),
            config: config,
        }
    }
    fn run(&mut self) {
        loop {
            lselect!{
                msg = self.rx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return
                            }
                        },
                        Err(e) => {
                            println!("WriterWorker Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        }
    }
    fn handle_msg(&mut self, msg: WriterThreadMsg) -> bool {
        match msg {
            WriterThreadMsg::SendRaw(raw) => {
                println!(">> (raw) {}", raw);
                self.stream.write(raw.into_bytes().as_slice());
            },
            WriterThreadMsg::Send(rpl) => {
                self.data.server_name = self.config.get_server_name().unwrap();
                let raw = rpl.raw(&mut self.data);
                println!(">> {} -- from {:?}", raw, rpl);
                self.stream.write(format!("{}\r\n", raw).into_bytes().as_slice());
            },
            WriterThreadMsg::UpdateNick(nick) => {
                self.data.nick = nick
            }
        };
        false
    }
}
