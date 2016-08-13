use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type ConfigThread = Sender<ConfigThreadMsg>;

pub enum ConfigThreadMsg {
    GetServerName(Sender<String>),
    GetClientBindAddr(Sender<String>),
    GetServerBindAddr(Sender<String>),
}

#[derive(Clone)]
pub struct Config {
    thread: ConfigThread,
}

impl Config {
    pub fn new(thread: ConfigThread) -> Self {
        Config{ thread: thread }
    }

    pub fn get_server_name(&self) -> Result<String> {
        Ok(try!(req_rep!(self.thread, ConfigThreadMsg::GetServerName => ())))
    }

    pub fn get_client_bind_addr(&self) -> Result<String> {
        Ok(try!(req_rep!(self.thread, ConfigThreadMsg::GetClientBindAddr => ())))
    }

    pub fn get_server_bind_addr(&self) -> Result<String> {
        Ok(try!(req_rep!(self.thread, ConfigThreadMsg::GetServerBindAddr => ())))
    }
}
