extern crate net;
extern crate channel;

pub fn run() {
    let directory = channel::DirectoryThreadFactory::new();
    net::run(directory);
}
