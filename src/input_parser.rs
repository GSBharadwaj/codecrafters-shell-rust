use crate::input_parser::State::{Space, Plain, SingleQuote, Default, DoubleQuote, Escape};
use crate::input_parser::TokenType::{WhiteSpace, Str};

enum State {
    Default,
    Space,
    Plain,
    SingleQuote,
    DoubleQuote,
    Escape,
}

#[derive(PartialEq)]
enum TokenType {
    WhiteSpace,
    Str,
}

struct Token {
    value: String,
    token_type: TokenType
}


pub fn parse(input: &str) -> Vec<String> {
    let mut char_peek = input.chars().peekable();

    let mut state: State = Default;
    let mut tokens = Vec::new();
    let mut token_buffer = String::new();

    while char_peek.peek().is_some() {
        let x =
        match char_peek.next()  {
            Some(y) => y,
            None => break
        };

        if x == '\n' {
            continue
        }
        match state {
            Default => {
                if x.is_whitespace() {
                    state = Space
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
            Space => {
                if x.is_whitespace() {
                    continue;
                } else if is_single_quote(&x) {
                    tokens.push(whitespace_token());
                    token_buffer.clear();
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    tokens.push(whitespace_token());
                    token_buffer.clear();
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    tokens.push(whitespace_token());
                    token_buffer.clear();
                    state = Escape;
                } else {
                    tokens.push(whitespace_token());
                    token_buffer.clear();
                    state = Plain;

                    token_buffer.push(x);
                }
            }
            Plain => {
                if x.is_whitespace() {
                    tokens.push(create_token(token_buffer.as_str()));
                    token_buffer.clear();
                    state = Space
                } else if is_single_quote(&x) {
                    tokens.push(create_token(token_buffer.as_str()));
                    token_buffer.clear();
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    tokens.push(create_token(token_buffer.as_str()));
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
                    tokens.push(create_token(token_buffer.as_str()));
                    token_buffer.clear();
                    state = Default
                } else {
                    token_buffer.push(x)
                }
            }
            DoubleQuote => {
                if is_double_quote(&x) {
                    tokens.push(create_token(token_buffer.as_str()));
                    token_buffer.clear();
                    state = Default
                } else {
                    token_buffer.push(x)
                }
            }
            Escape => {
                token_buffer.push(x); //All special meanings including "escape" fall apart
                state = Default
            }
        }
    }
    if !token_buffer.is_empty() {
        match state {
            Space => {}
            _ => {
                tokens.push(create_token(token_buffer.as_str()));
                token_buffer.clear()
            }
        }
    }

    let res = tokens_to_strings(&tokens);
    res
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
        match tokens[i].token_type {
            WhiteSpace => {
                if !buffer.is_empty() {
                    result.push(buffer.as_str().to_string());
                    buffer.clear();
                }
            }
            Str => {
                buffer.push_str(tokens[i].value.as_str());
            }
        }
    }
    if !buffer.is_empty() {
        result.push(buffer.as_str().to_string());
    }
    result
}

fn whitespace_token() -> Token {
    Token {
        value: "".to_string(),
        token_type: WhiteSpace
    }
}

fn create_token(token_string: &str) -> Token {
    Token {
        value: token_string.to_string(),
        token_type: Str
    }
}