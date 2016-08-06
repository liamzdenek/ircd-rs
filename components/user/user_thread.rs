use std::sync::mpsc::{channel, Receiver};
use std::thread;

use user_traits::{UserThread, UserThreadMsg};

pub trait UserThreadFactory {
    fn new() -> Self;
}

impl UserThreadFactory for UserThread {
    fn new() -> UserThread {
        let (tx,rx) = channel();
        println!("CREATING USER THREAD FACTORY INSTANCE");
        thread::Builder::new().name("UserThread".to_string()).spawn(move || {
            UserWorker::new(rx).run();
            println!("ENDING USER THREAD FACTORY INSTANCE");
        });

        tx
    }
}

pub struct UserWorker {
    rx: Receiver<UserThreadMsg>,
}

impl UserWorker {
    fn new(rx: Receiver<UserThreadMsg>) -> Self {
        UserWorker{ rx: rx }
    }

    fn run(&mut self) {
        println!("user worker starting");
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
                            println!("Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        }
    }

    fn handle_msg(&mut self, msg: UserThreadMsg) -> bool {
        println!("got msg: {:?}", msg);
        return false;
    }
}
