use std::net::TcpStream;
use std::io::{BufReader,BufRead};

use linefsm::LineFSM;
use net_traits::*;
use usercomponent::UserThreadFactory;
use user_traits::{User as TUser};
use channel_traits::DirectoryThread;

use super::{WriterThreadFactory};

pub struct User {
    stream: TcpStream,
    directory: DirectoryThread,
    buf: BufReader<TcpStream>,
}

impl User {
    pub fn new(stream: TcpStream, directory: DirectoryThread) -> Self{
        User{
            buf: BufReader::new(stream.try_clone().unwrap()),
            stream: stream,
            directory: directory,
        }
    }

    pub fn run(&mut self) -> Result<()>{
        let mut fsm = LineFSM::new();
        let writer = Writer::new(WriterThreadFactory::new(self.stream.try_clone().unwrap()));
        let user = TUser::new(UserThreadFactory::new(writer, self.directory.clone()));
        loop {
            let line = try!(self.read_line());
            let cmd = try!(fsm.handle_line(line));
            match user.send_command(cmd) {
                Err(e) => {
                    println!("error: {:?}", e);
                    return Err(Error::UserError);
                }
                _ => {}
            }
        }
        unreachable!{}
    }

    pub fn read_line(&mut self) -> Result<String>{
        let mut str = String::new();
        try!(self.buf.read_line(&mut str));
        match self.stream.take_error() {
            Ok(Some(err)) => try!(Err(err)),
            _ => {}
        }
        Ok(str)
    }
}
