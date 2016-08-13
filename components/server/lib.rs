#![feature(plugin)]
#![feature(serde_macros)]
#![feature(custom_derive)]
#![feature(mpsc_select)]
#[macro_use]
extern crate util;
extern crate user_traits;
extern crate net_traits;
extern crate channel_traits;
extern crate server_traits;

extern crate serde;
extern crate serde_yaml;

#[derive(Debug, Deserialize)]
pub struct ConfigData {
    server_name: String,
    client_bind_addr: String,
    server_bind_addr: String,
}

pub mod server_thread;
pub mod config_thread;

pub use server_thread::*;
pub use config_thread::*;
