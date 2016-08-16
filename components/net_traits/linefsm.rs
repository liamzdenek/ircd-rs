use std::string::String;

#[derive(Debug)]
pub struct ParsedCommand {
    //prefix: Option<String>,
    pub command: String,
    pub params: Vec<String>,
    pub trailing: Vec<String>,
}

