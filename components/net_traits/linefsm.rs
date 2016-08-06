use std::string::String;
use error::*;

#[derive(Debug)]
pub struct ParsedCommand {
    //prefix: Option<String>,
    command: String,
    params: Vec<String>,
    trailing: Vec<String>,
}

