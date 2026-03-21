use std::env;
use std::path::PathBuf;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Context;
use crate::input_parser::{tokenize, Token};
use crate::input_parser::Token::Str;
use crate::trie::trie::Trie;

pub struct ReadLineHelper {
    command_trie: Trie,
    cur_dir_lister: Box<dyn Fn(&str) -> Vec<String> + Send + Sync>,
}

impl ReadLineHelper {
    pub fn new<F>(commands: Vec<String>, cur_dir_lister: F) -> Self
    where
        F: Fn(&str) -> Vec<String> + Send + Sync + 'static
    {
        Self {
            command_trie: Trie::new(commands),
            cur_dir_lister: Box::new(cur_dir_lister),
        }
    }
}

impl Completer for ReadLineHelper {

    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let input = &line[..pos];
        let tokens: Vec<Token> = tokenize(input);
        if tokens.len() == 1 { //command
            let mut matches: Vec<String> = self.command_trie.prefix_search(input);

            matches.sort();
            matches.dedup();

            if matches.len() == 1 {
                return Ok((0, vec![format!("{} ", &matches[0])]));
            }

            Ok((0, matches))
        } else if tokens.len() >= 2 {
            match &tokens[..] {
                [_prefix @ .., Str((x, _)), Str((_, last_arg))] => {
                    let directory_to_search =
                    if last_arg.starts_with("/") || last_arg.starts_with("~./") {
                        let prefix_path = PathBuf::from(last_arg);
                        if let Some(parent) = prefix_path.parent() {
                            parent.to_path_buf()
                        } else {
                            return Ok((x.to_owned(), vec![]))
                        }
                    } else {
                        env::current_dir()?
                    };
                    let list_of_files = (self.cur_dir_lister)(directory_to_search.to_str().unwrap());
                    let temp_trie = Trie::new(list_of_files);
                    let matches = temp_trie.prefix_search(last_arg.as_ref());

                    if matches.len() == 1 {
                        return Ok((x.to_owned() + 1, vec![format!("{} ", &matches[0])]));
                    }

                    Ok((x.to_owned() + 1, matches))
                }
                _ => Ok((0, Vec::new()))
            }
        } else {
            Ok((0, Vec::new()))
        }
    }

}

impl Hinter for ReadLineHelper { type Hint = String; }
impl Highlighter for ReadLineHelper {}
impl Validator for ReadLineHelper {}
impl rustyline::Helper for ReadLineHelper {}
