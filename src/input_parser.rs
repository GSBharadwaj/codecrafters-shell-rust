use std::collections::VecDeque;
use crate::input_parser::State::{SingleQuote, Default, DoubleQuote, Escape};
use crate::input_parser::Token::{Str, Redir, RedirErr, Pipe};
use crate::models::ShellCmd;

enum State {
    Default,
    SingleQuote,
    DoubleQuote,
    Escape,
}

#[derive(PartialEq)]
pub enum Token {
    Redir(/*append?*/ bool),
    RedirErr(/*append?*/ bool),
    Pipe,
    Str((usize, String)),
}

pub fn parse(input: &str) -> Result<Vec<ShellCmd>, String> {
    let tokens = tokenize(input);
    parse_tokens(&tokens)
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut char_peek = input.chars().enumerate().peekable();

    let mut state: State = Default;
    let mut tokens = Vec::new();
    let mut token_buffer = String::new();
    let mut state_stack: VecDeque<State> = VecDeque::new();

    while let Some((idx, x)) = char_peek.next() {
        match state {
            Default => {
                if x.is_whitespace() {
                    if !token_buffer.is_empty() {
                        tokens.push(Str((idx, token_buffer.to_owned())));
                        token_buffer.clear();
                    }
                } else if is_single_quote(&x) {
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    state = Escape;
                } else if x == '>' {
                    match (char_peek).peek() {
                        Some((_, '>')) => {
                            char_peek.next();
                            tokens.push(Redir(true))
                        }
                        _ => { tokens.push(Redir(false)) }
                    }
                } else if x == '1' {
                    match char_peek.peek() {
                        Some((_, '>')) => { continue },
                        _ => { token_buffer.push(x) }
                    }
                } else if x == '2' {
                    match char_peek.peek() {
                        Some((_, '>')) => {
                            char_peek.next();
                            match char_peek.peek() {
                                Some((_,'>')) => {
                                    char_peek.next();
                                    tokens.push(RedirErr(true))
                                }
                                _ => {
                                    tokens.push(RedirErr(false));
                                }
                            }
                            state = Default;
                        },
                        _ => {
                            token_buffer.push(x)
                        }
                    }
                } else if x == '|' {
                    if !token_buffer.is_empty() {
                        tokens.push(Str((idx, token_buffer.to_owned())));
                        token_buffer.clear();
                    }
                    tokens.push(Pipe)
                } else {
                    token_buffer.push(x)
                }
            }
            SingleQuote => {
                if is_single_quote(&x) {
                    state = Default
                } else {
                    token_buffer.push(x)
                }
            }
            DoubleQuote => {
                if is_double_quote(&x) {
                    state = Default
                } else if is_backslash(&x) {
                    match char_peek.peek() {
                        Some((_, '\\')) |
                        Some((_ ,'"')) => {
                            state = Escape;
                            state_stack.push_back(DoubleQuote);
                        }
                        _ => token_buffer.push(x) //treat it as literal
                    }
                } else {
                    token_buffer.push(x);
                }
            }
            Escape => {
                token_buffer.push(x);
                state = state_stack.pop_back().unwrap_or(Default);
            }
        }
    }
    let last_string = token_buffer.as_str();
    if !last_string.is_empty() {
        tokens.push(Str((input.len()-1, last_string.to_string())));
        token_buffer.clear()
    }
    tokens
}

fn is_single_quote(x: &char) -> bool {
    *x == '\''
}

fn is_backslash(x: &char) -> bool {
    *x == '\\'
}

fn is_double_quote(x: &char) -> bool {
    *x == '"'
}


pub fn parse_tokens(tokens: &Vec<Token>) -> Result<Vec<ShellCmd>, String> {
    let mut args = Vec::new();
    let mut cmds = Vec::new();

    let mut redirection_path = None;
    let mut redirection_append = false;
    let mut err_redirection_path = None;
    let mut err_redirection_append = false;
    let mut token_iter = tokens.iter().peekable();

    while let Some(token) = token_iter.next() {
        match token {
            RedirErr (append)=> {
                match token_iter.next() {
                    Some(Str((_, x))) => {
                        err_redirection_path = Some(x.to_owned());
                        err_redirection_append = *append;
                    },
                    Some(RedirErr(_)) => { return Err("shell: unexpected token 2".to_string()) },
                    Some(Redir(_)) => { return Err("shell: unexpected token >".to_string()) },
                    Some(Pipe) => { return Err("shell: unexpected token |".to_string()) }
                    None => { return Err("shell: unexpected token \\\n".to_string()); },
                }
            }
            Redir(append) => {
                match token_iter.next() {
                    Some(Str((_, x))) => {
                        redirection_path = Some(x.to_owned());
                        redirection_append = *append;
                    },
                    Some(RedirErr(_)) => { return Err("shell: unexpected token 2".to_string()) },
                    Some(Redir(_)) => { return Err("shell: unexpected token >".to_string()) },
                    Some(Pipe) => { return Err("shell: unexpected token |".to_string()) }
                    None => { return Err("shell: unexpected token \\\n".to_string()); },
                }
                //Some fancy rust to skip whitespaces
            }
            Str((_, token)) => {
                args.push(token.to_string());
            }
            Pipe => {
                cmds.push(ShellCmd {
                    args,
                    redirection_path,
                    redirection_append,
                    err_redirection_path,
                    err_redirection_append,
                });

                args = Vec::new();
                redirection_path = None;
                redirection_append = false;
                err_redirection_path = None;
                err_redirection_append = false;
            }
        }
    }
    cmds.push(ShellCmd {
        args,
        redirection_path,
        redirection_append,
        err_redirection_path,
        err_redirection_append,
    });

    Ok(cmds)
}

