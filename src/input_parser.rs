use std::collections::VecDeque;
use crate::input_parser::State::{Plain, SingleQuote, Default, DoubleQuote, Escape};
use crate::input_parser::Token::{WhiteSpace, Str};

enum State {
    Default,
    Plain,
    SingleQuote,
    DoubleQuote,
    Escape,
}

#[derive(PartialEq)]
enum Token {
    WhiteSpace,
    Str(String),
}

pub fn parse(input: &str) -> Vec<String> {
    let mut char_peek = input.chars().peekable();

    let mut state: State = Default;
    let mut tokens = Vec::new();
    let mut token_buffer = String::new();
    let mut state_stack: VecDeque<State> = VecDeque::new();

    while let Some(x) = char_peek.next() {
        match state {
            Default => {
                if x.is_whitespace() {
                    tokens.push(WhiteSpace)
                } else if is_single_quote(&x) {
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    state = Escape;
                } else {
                    state = Plain;
                    token_buffer.push(x)
                }
            }
            Plain => {
                if x.is_whitespace() {
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();

                    tokens.push(WhiteSpace);
                    state = Default
                } else if is_single_quote(&x) {
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    state = Escape
                } else {
                    token_buffer.push(x)
                }
            }
            SingleQuote => {
                if is_single_quote(&x) {
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();
                    state = Default
                } else {
                    token_buffer.push(x)
                }
            }
            DoubleQuote => {
                if is_double_quote(&x) {
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();
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

    tokens_to_strings(&tokens)
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

fn tokens_to_strings(tokens: &Vec<Token>) -> Vec<String> {
    let mut buffer = String::new();
    let mut result = Vec::new();

    for i in 0..tokens.len() {
        match &tokens[i] {
            WhiteSpace => {
                if !buffer.is_empty() {
                    result.push(buffer.as_str().to_string());
                    buffer.clear();
                }
            }
            Str(token) => {
                buffer.push_str(token.as_str());
            }
        }
    }
    if !buffer.is_empty() {
        result.push(buffer.as_str().to_string());
    }
    result
}
