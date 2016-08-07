use std::sync::mpsc::{channel, Receiver};
use std::thread;
use channel_traits::*;

pub trait DirectoryThreadFactory {
    fn new() -> Self;
}

impl DirectoryThreadFactory for DirectoryThread {
    fn new() -> DirectoryThread {
        let (tx,rx) = channel();
        thread::Builder::new().name("DirectoryThread".to_string()).spawn(move || {
            DirectoryWorker::new(rx).run();
        });
        tx
    }
}


pub struct DirectoryWorker {
    rx: Receiver<DirectoryThreadMsg>,
}

impl DirectoryWorker {
    fn new(rx: Receiver<DirectoryThreadMsg>) -> Self {
        DirectoryWorker{ rx: rx }
    }

    fn run(&mut self) {
        loop {
            lselect!{
                msg = self.rx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return;
                            }
                        }
                        Err(e) => {
                            println!("DirectoryThread Got error: {:?}", e);
                        }
                    }
                },
            };
        }
    }

    fn handle_msg(&mut self, msg: DirectoryThreadMsg) -> bool{
        println!("Directory Thread got msg: {:?}", msg);
        return false;
    }
}
