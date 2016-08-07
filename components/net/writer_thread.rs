use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;
use std::io::Write;


use net_traits::{WriterThread,WriterThreadMsg};

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
}

impl WriterWorker {
    fn new(stream: TcpStream, rx: Receiver<WriterThreadMsg>) -> Self {
        WriterWorker{
            stream: stream,
            rx: rx,
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
            WriterThreadMsg::Send(raw) => {
                println!("Send raw: {:?}", raw);
                self.stream.write(raw.into_bytes().as_slice());
            }
        };
        false
    }
}
