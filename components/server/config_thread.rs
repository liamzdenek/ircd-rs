use std::sync::mpsc::{channel, Receiver};
use std::net::TcpStream;
use std::thread;
use std::path::Path; 
use server_traits::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use serde_yaml;
use std::str;
use super::ConfigData;
use std::collections::BTreeMap;

pub fn parse_config(file: &Path) -> ConfigData {
    // TODO: not hardcode this
    let mut f = File::open(file).unwrap();
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).unwrap();

    println!("Read file: {:?}", buffer);

    // custom derive deserialize seems to be broken, TODO: make that work
    let mut data: BTreeMap<String, String> = BTreeMap::new();


    data = serde_yaml::from_str(str::from_utf8(&buffer).unwrap()).unwrap();
    println!("Data: {:?}", data);

    ConfigData{
        server_name: data.get("server_name").unwrap().to_owned(),
        client_bind_addr: data.get("client_bind_addr").unwrap().to_owned(),
        server_bind_addr: data.get("server_bind_addr").unwrap().to_owned(),
        server_pass: data.get("server_pass").unwrap().to_owned(),
    }
}

pub trait ConfigThreadFactory {
    fn new(ConfigData) -> Self;
}

impl ConfigThreadFactory for ConfigThread {
    fn new(data: ConfigData) -> ConfigThread {
        let (tx, rx) = channel();
        thread::Builder::new().name("ConfigThread".to_string()).spawn(move || {
            ConfigWorker::new(rx, data).run();
        });
        tx
    }
}

pub struct ConfigWorker {
    rx: Receiver<ConfigThreadMsg>,
    data: ConfigData
}

impl ConfigWorker {
    fn new(rx: Receiver<ConfigThreadMsg>, data: ConfigData) -> Self{
        ConfigWorker{
            rx: rx,
            data: data,
        }
    }

    fn run(&mut self) {
        loop {
            lselect!{
                msg = self.rx => {
                    match msg {
                        Ok(msg) => {
                            if self.handle_msg(msg) {
                                return
                            }
                        },
                        Err(e) => {
                            println!("UserThread Got error: {:?}", e);
                            return;
                        }
                    }
                },
            }
        }
    }

    fn handle_msg(&mut self, msg: ConfigThreadMsg) -> bool {
        match msg {
            ConfigThreadMsg::GetServerName(s) => s.send(self.data.server_name.clone()),
            ConfigThreadMsg::GetClientBindAddr(s) => s.send(self.data.client_bind_addr.clone()),
            ConfigThreadMsg::GetServerBindAddr(s) => s.send(self.data.server_bind_addr.clone()),
            ConfigThreadMsg::GetServerPass(s) => s.send(self.data.server_pass.clone()),
        };
        false
    }
}
