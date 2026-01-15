use std::collections::VecDeque;
use crate::input_parser::State::{SingleQuote, Default, DoubleQuote, Escape};
use crate::input_parser::Token::{Str, Redir, RedirErr};
use crate::models::ShellCmd;

enum State {
    Default,
    SingleQuote,
    DoubleQuote,
    Escape,
}

#[derive(PartialEq)]
enum Token {
    Redir,
    RedirErr,
    Str(String),
}

pub fn parse(input: &str) -> Result<ShellCmd, String> {
    let mut char_peek = input.chars().peekable();

    let mut state: State = Default;
    let mut tokens = Vec::new();
    let mut token_buffer = String::new();
    let mut state_stack: VecDeque<State> = VecDeque::new();

    while let Some(x) = char_peek.next() {
        match state {
            Default => {
                if x.is_whitespace() {
                    if !token_buffer.is_empty() {
                        tokens.push(Str(token_buffer.to_owned()));
                        token_buffer.clear();
                    }
                } else if is_single_quote(&x) {
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    state = Escape;
                } else if is_redirect(&x) {
                    tokens.push(Redir)
                } else if x == '1' {
                    match char_peek.peek() {
                        Some('>') => {
                            continue
                        },
                        _ => {
                            token_buffer.push(x)
                        }
                    }
                } else if x == '2' {
                    match char_peek.next() {
                        Some('>') => {
                            tokens.push(RedirErr);
                            state = Default;
                        },
                        Some(c) => {
                            token_buffer.push(x);
                            token_buffer.push(c);
                        }
                        None => {
                            token_buffer.push(x)
                        }
                    }
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
                        Some('\\') |
                        Some('"') => {
                            state = Escape;
                            state_stack.push_back(DoubleQuote);
                        }
                        _ => token_buffer.push(x)//treat it as literal
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
    let last_string = token_buffer.as_str().trim();
    if !last_string.is_empty() {
        tokens.push(Str(last_string.to_string()));
        token_buffer.clear()
    }

    parse_tokens(&tokens)
}

fn is_redirect(x: &char) -> bool {
    *x == '>'
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


fn parse_tokens(tokens: &Vec<Token>) -> Result<ShellCmd, String> {
    let mut args = Vec::new();

    let mut redirection_path = None;
    let mut err_redirection_path = None;
    let mut token_iter = tokens.iter().peekable();

    while let Some(token) = token_iter.next() {
        match token {
            RedirErr => {
                match token_iter.next() {
                    Some(Str(x)) => err_redirection_path = Some(x.to_owned()),
                    Some(RedirErr) => { return Err("shell: unexpected token 2".to_string()) },
                    Some(Redir) => { return Err("shell: unexpected token >".to_string()) },
                    None => { return Err("shell: unexpected token \\\n".to_string()); },
                }
            }
            Redir => {

                match token_iter.next() {
                    Some(Str(x)) => redirection_path = Some(x.to_owned()),
                    Some(RedirErr) => { return Err("shell: unexpected token 2".to_string()) },
                    Some(Redir) => { return Err("shell: unexpected token >".to_string()) },
                    None => { return Err("shell: unexpected token \\\n".to_string()); },
                }
                //Some fancy rust to skip whitespaces
            }
            Str(token) => {
                args.push(token.to_string());
            }
        }
    }


    Ok(ShellCmd {
        args,
        redirection_path,
        err_redirection_path,
    })
}

