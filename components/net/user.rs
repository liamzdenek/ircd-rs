use std::net::TcpStream;
use std::io::{BufReader,BufRead,Error as IoError};

use linefsm::LineFSM;
use net_traits::error::*;

pub struct User {
    stream: TcpStream,
    buf: BufReader<TcpStream>,
}

impl User {
    pub fn new(stream: TcpStream) -> Self{
        User{
            buf: BufReader::new(stream.try_clone().unwrap()),
            stream: stream,
        }
    }

    pub fn run(&mut self) -> Result<()>{
        let mut fsm = LineFSM::new();
        loop {
            let line = try!(self.read_line());
            let msg = try!(fsm.handle_line(line));
            println!("Got msg: {:?}", msg);
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
