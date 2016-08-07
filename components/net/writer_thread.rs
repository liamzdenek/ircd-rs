use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;
use std::io::Write;

use net_traits::*;


pub trait WriterThreadFactory {
    fn new(TcpStream) -> Self;
}

impl WriterThreadFactory for WriterThread {
    fn new(stream: TcpStream) -> WriterThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("WriterThread".to_string()).spawn(move || {
            WriterWorker::new(stream, rx).run();
        });
        tx
    }
}

pub struct WriterWorker {
    stream: TcpStream,
    rx: Receiver<WriterThreadMsg>,
    data: WriterData,
}

impl WriterWorker {
    fn new(stream: TcpStream, rx: Receiver<WriterThreadMsg>) -> Self {
        WriterWorker{
            stream: stream,
            rx: rx,
            data: Default::default(),
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
                            println!("UserThread Got error: {:?}", e);
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
                let raw = rpl.raw(&self.data);
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
