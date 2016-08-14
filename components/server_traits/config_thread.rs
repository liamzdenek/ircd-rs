use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type ConfigThread = Sender<ConfigThreadMsg>;

pub enum ConfigThreadMsg {
    GetServerName(Sender<String>),
    GetClientBindAddr(Sender<String>),
    GetServerBindAddr(Sender<String>),
    GetServerPass(Sender<String>),
}

#[derive(Clone)]
pub struct Config {
    thread: ConfigThread,
}

impl Config {
    pub fn new(thread: ConfigThread) -> Self {
        Config{ thread: thread }
    }

    pub fn get_server_name(&self) -> String {
        req_rep!(self.thread, ConfigThreadMsg::GetServerName => ()).unwrap()
    }

    pub fn get_client_bind_addr(&self) -> String{
       req_rep!(self.thread, ConfigThreadMsg::GetClientBindAddr => ()).unwrap()
    }

    pub fn get_server_bind_addr(&self) -> String {
        req_rep!(self.thread, ConfigThreadMsg::GetServerBindAddr => ()).unwrap()
    }

    pub fn get_server_pass(&self) -> String {
        req_rep!(self.thread, ConfigThreadMsg::GetServerPass => ()).unwrap()
    }
}
