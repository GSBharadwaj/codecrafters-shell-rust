use std::collections::VecDeque;
use std::mem::take;
use crate::input_parser::State::{Plain, SingleQuote, Default, DoubleQuote, Escape};
use crate::input_parser::Token::{WhiteSpace, Str, Redir};
use crate::models::ShellCmd;

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
    Redir,
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
                    tokens.push(WhiteSpace)
                } else if is_single_quote(&x) {
                    state = SingleQuote
                } else if is_double_quote(&x) {
                    state = DoubleQuote
                } else if is_backslash(&x) {
                    state = Escape;
                } else if is_redirect(&x) {
                    tokens.push(Redir)
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
                } else if is_redirect(&x){
                    tokens.push(Str(token_buffer.to_owned()));
                    token_buffer.clear();

                    tokens.push(Redir);

                    state = Default
                } else if is_double_quote(&x) {

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

struct ParserState {
    args: Vec<String>,
    redirection_path: Option<String>,
    is_redirection_token: bool,
}

fn parse_tokens(tokens: &Vec<Token>) -> Result<ShellCmd, String> {
    let mut parser_state = ParserState {
        args: Vec::new(),
        redirection_path: None,
        is_redirection_token: false,
    };

    let mut buffer = String::new();
    let mut token_iter = tokens.iter().peekable();
    while let Some(token) = token_iter.next() {
        match token {
            WhiteSpace => {
                parser_state  = flush_buffer(take(&mut buffer), parser_state)?;
            }
            Redir => {
                parser_state = flush_buffer(take(&mut buffer), parser_state)?;
                parser_state.is_redirection_token = true;

                //Some fancy rust to skip whitespaces
                while token_iter.next_if(|&t| matches!(t, WhiteSpace)).is_some() { }
            }
            Str(token) => {
                buffer.push_str(token.as_str());
            }
        }
    }

    parser_state = flush_buffer(take(&mut buffer), parser_state)?;

    Ok(ShellCmd {
        args: parser_state.args,
        redirection_path: parser_state.redirection_path,
    })
}

fn flush_buffer(buffer: String, mut state: ParserState) -> Result<ParserState, String> {
    if buffer.is_empty() {
        return if state.is_redirection_token {
            Err("shell: expected file path".to_string())
        } else { Ok(state) };
    }
    if state.is_redirection_token {
        state.redirection_path = Some(buffer);
        state.is_redirection_token = false; // Reset after successfully finding the path
    } else {
        state.args.push(buffer);
    }
    Ok(state)
}
