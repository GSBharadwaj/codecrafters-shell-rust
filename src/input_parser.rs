use crate::input_parser::TokenType::{WhiteSpace, Plain, SingleQuote, Default};

#[derive(PartialEq)]
enum TokenType {
    Default,
    WhiteSpace,
    Plain,
    SingleQuote,
}

struct Token {
    value: String,
    token_type: TokenType
}


pub fn parse(input: &str) -> Vec<String> {
    let mut char_peek = input.chars().peekable();

    let mut state: TokenType = Default;
    let mut tokens = Vec::new();
    let mut token_buffer = String::new();

    while char_peek.peek().is_some() {
        match char_peek.next() {
            Some(x) => {
                if x.is_whitespace() {
                    match state {
                        Plain => {
                            add_token(&mut tokens, token_buffer.as_str(), Plain);
                            token_buffer.clear();
                            state = WhiteSpace;
                        }
                        SingleQuote => {
                            token_buffer.push(x)
                        }
                        Default => { state = WhiteSpace }
                        WhiteSpace => continue
                    }
                } else if is_single_quote(&x) {
                    match state {
                        WhiteSpace => {
                            add_token(&mut tokens, "", WhiteSpace);
                            token_buffer.clear();
                            state = SingleQuote;
                        }
                        Plain => {
                            add_token(&mut tokens, token_buffer.as_str(), Plain);
                            token_buffer.clear();
                            state = SingleQuote
                        },
                        SingleQuote => {
                            add_token(&mut tokens, token_buffer.as_str(), SingleQuote);
                            token_buffer.clear();
                            state = Default
                        }
                        Default => {
                            state = SingleQuote
                        },
                    }
                } else {
                    match state {
                        WhiteSpace => {
                            add_token(&mut tokens, "", WhiteSpace);
                            token_buffer.push(x);
                            state = Plain
                        }
                        Default => {
                            token_buffer.push(x);
                            state=Plain
                        }
                        _ => {token_buffer.push(x)}
                    }
                }
            }
            None => break
        }
    }
    let res = tokens_to_strings(&tokens);
    res
}

fn is_single_quote(x: &char) -> bool {
    *x == '\''
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
            Plain => {
                buffer.push_str(tokens[i].value.as_str());
            }
            SingleQuote => {
                buffer.push_str(tokens[i].value.as_str());
            }
            Default => continue
        }
    }
    if !buffer.is_empty() {
        result.push(buffer.as_str().to_string());
    }
    result
}

fn add_token(res: &mut Vec<Token>, token_string: &str, token_type: TokenType) {
    let token = Token {
        value: token_string.to_string(),
        token_type
    };

    res.push(token);
}