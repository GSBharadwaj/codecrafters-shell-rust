use crate::input_parser::TokenType::{Default, Plain};

#[derive(PartialEq)]
enum TokenType {
    Default,
    Plain,
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
                            state = Default;
                        }
                        Default => continue
                    }
                } else {
                    state = Plain;
                    token_buffer.push(x)
                }
            }
            None => break
        }
    }
    let res = tokens_to_strings(&tokens);
    if res.is_empty() {
        let mut new_res = Vec::new();
        new_res.push("pwd".to_string());
       new_res
    } else {
        res
    }
}

fn tokens_to_strings(tokens: &Vec<Token>) -> Vec<String> {
    tokens.into_iter().filter(|t| match t.token_type {Default => false, _ => true}).map(|t| t.value.clone()).collect()
}

fn add_token(res: &mut Vec<Token>, token_string: &str, token_type: TokenType) {
    let token = Token {
        value: token_string.to_string(),
        token_type
    };

    res.push(token);
}