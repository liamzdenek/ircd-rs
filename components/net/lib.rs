#![feature(mpsc_select)]
#[macro_use]
extern crate util;
extern crate net_traits;
extern crate user as usercomponent;
extern crate user_traits;

use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod user;
pub mod linefsm;
pub mod writer_thread;

//pub use user::*;
pub use linefsm::*;
pub use writer_thread::*;

use user::User;

pub fn run() {
    println!("hello world");
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(e) => panic!(e),
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream);
                });
            }
        }
    }
}

fn handle_client(stream: TcpStream) {
    let err = User::new(stream).run();
    println!("Connection ended with err: {:?}", err);
}
