use crate::input_parser::TokenType::{WhiteSpace, Plain, SingleQuote, Default, DoubleQuote};

#[derive(PartialEq)]
enum TokenType {
    Default,
    WhiteSpace,
    Plain,
    SingleQuote,
    DoubleQuote,
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
                if x == '\n' {
                    continue
                }
                if x.is_whitespace() {
                    match state {
                        Plain => {
                            add_token(&mut tokens, token_buffer.as_str(), Plain);
                            token_buffer.clear();
                            state = WhiteSpace;
                        }
                        DoubleQuote => {
                            token_buffer.push(x)
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
                        DoubleQuote => {
                            token_buffer.push(x);
                        }
                        SingleQuote => {
                            add_token(&mut tokens, token_buffer.as_str(), SingleQuote);
                            token_buffer.clear();
                            state = Default
                        }
                        Default => {
                            state = SingleQuote
                        },
                    }
                } else if is_double_quote(&x) {
                    match state {
                        Default => {
                            state = DoubleQuote
                        }
                        WhiteSpace => {
                            add_token(&mut tokens, "", WhiteSpace);
                            token_buffer.clear();
                            state = DoubleQuote
                        }
                        Plain => {
                            add_token(&mut tokens, token_buffer.as_str(), Plain);
                            token_buffer.clear();
                            state = DoubleQuote;
                        }
                        SingleQuote => {
                            token_buffer.push(x);
                        }
                        DoubleQuote => {
                            add_token(&mut tokens, token_buffer.as_str(), DoubleQuote);
                            token_buffer.clear();
                            state = Default
                        }
                    }
                }
                else {
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
    if !token_buffer.is_empty() {
        match state {
            Plain | SingleQuote => {
                add_token(&mut tokens, token_buffer.as_str(), state);
                token_buffer.clear()
            }
            _ => {}
        }
    }

    let res = tokens_to_strings(&tokens);
    res
}

fn is_single_quote(x: &char) -> bool {
    *x == '\''
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
            Plain => {
                buffer.push_str(tokens[i].value.as_str());
            }
            SingleQuote => {
                buffer.push_str(tokens[i].value.as_str());
            }
            DoubleQuote => {
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