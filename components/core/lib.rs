extern crate net;
extern crate channel;
extern crate channel_traits;
extern crate server;
extern crate server_traits;

use std::path::Path;
use std::env;

pub fn run() {
    let arg = env::args().nth(1);
    if arg.is_none() {
        println!("Usage: server.bin [config.yaml]");
        return;
    }
    let config = server_traits::Config::new(server::ConfigThreadFactory::new(server::parse_config(Path::new(&arg.unwrap()))));
    let directory = channel_traits::Directory::new(channel::DirectoryThreadFactory::new());
    net::run(directory, config);
}
