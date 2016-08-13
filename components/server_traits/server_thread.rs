use std::sync::mpsc::{channel, Sender};
use util::*;
use super::Result;

pub type ServerThread = Sender<ServerThreadMsg>;

pub enum ServerThreadMsg {

}

