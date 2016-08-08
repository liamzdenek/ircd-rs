extern crate net;
extern crate channel;
extern crate channel_traits;

pub fn run() {
    let directory = channel_traits::Directory::new(channel::DirectoryThreadFactory::new());
    net::run(directory);
}
