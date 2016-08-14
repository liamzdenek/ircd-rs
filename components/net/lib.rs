#![feature(mpsc_select)]
#[macro_use]
extern crate util;
extern crate net_traits;
extern crate user as usercomponent;
extern crate user_traits;
extern crate channel_traits;
extern crate server_traits;

use std::net::{TcpListener};
use std::thread;

pub mod user;
pub mod linefsm;
pub mod writer_thread;

//pub use user::*;
pub use linefsm::*;
pub use writer_thread::*;

use channel_traits::Directory;
use user::User;
use server_traits::Config;

pub fn run(directory: Directory, config: Config) {
    println!("hello world");
    let listener = TcpListener::bind(config.get_client_bind_addr().as_str()).unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(e) => panic!(e),
            Ok(stream) => {
                let directory_clone = directory.clone();
                let config_clone = config.clone();
                thread::spawn(move|| {
                    let err = User::new(stream, config_clone, directory_clone).run();
                    println!("Connection ended with err: {:?}", err);
                });
            }
        }
    }
}
