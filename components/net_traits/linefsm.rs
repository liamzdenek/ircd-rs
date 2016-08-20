use std::string::String;

#[derive(Debug)]
pub struct ParsedCommand {
    pub prefix: String,
    pub command: String,
    pub params: Vec<String>,
    pub trailing: Vec<String>,
}

