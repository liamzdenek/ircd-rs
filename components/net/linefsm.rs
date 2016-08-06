use std::string::String;
use net_traits::error::*;
use net_traits::ParsedCommand;

pub struct LineFSM {
}

impl LineFSM {
    pub fn new() -> Self {
        LineFSM{ }
    }

    pub fn handle_line(&mut self, line: String) -> Result<ParsedCommand> {
        self._handle_line(State::Ready{line: line})
    }

    fn _handle_line(&mut self, mut state: State) -> Result<ParsedCommand> {
        let mut prefix: Option<String> = None; 
        let mut command: String = "".into(); 
        let mut params = vec![];
        let mut trailing = vec![];
        loop {
            match state {
                State::Ready{mut line} => {
                    let line = line.trim_right();
                    println!("line: {:?}", line);

                    if line.chars().next() == Some(':') {
                        state = State::ParsePrefix{line: line.into()};
                        continue;
                    }

                    state = State::ParseCommand{line: line.into()}
                },
                State::ParsePrefix{line} => {
                    unimplemented!{}
                },
                State::ParseCommand{line} => {
                    let mut iter = line.split_whitespace();
                    match iter.next() {
                        Some(ref word) => {command = word.to_string()},
                        None => { return Err(Error::MalformedString); }
                    };
                    let mut remaining = Vec::with_capacity(iter.size_hint().0);
                    while let Some(ref word) = iter.next() {
                        remaining.push(word.to_string());
                    }
                    state = if remaining.len() > 0 {
                        State::ParseParams{remaining: remaining}
                    } else {
                        State::Complete
                    }
                },
                State::ParseParams{mut remaining} => {
                    params = vec![];
                    trailing = vec![];
                    let mut which = true;
                    for mut item in remaining.into_iter() {
                        if item.chars().next() == Some(':') {
                            item = item.split_at(1).1.to_owned();
                            which = false;
                        }
                        if which {
                            params.push(item);
                        } else {
                            trailing.push(item);
                        }
                    }
                    state = State::Complete
                },
                State::Complete => {
                    return Ok(ParsedCommand{
                        //prefix: prefix,
                        command: command,
                        params: params,
                        trailing: trailing,
                    })
                }
            }
        }
    }
}

enum State {
    // init states
    Ready{line: String}

    // parsing states
    ,ParsePrefix{line: String}
    ,ParseCommand{line: String}
    ,ParseParams{remaining: Vec<String>}

    // complete states
    ,Complete
}
