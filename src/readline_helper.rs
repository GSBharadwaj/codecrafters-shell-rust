use std::env;
use std::path::{PathBuf};
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

    fn get_dir_to_search(parent: &str) -> PathBuf {
        if parent.is_empty() {
            return env::current_dir().unwrap()
        }
        PathBuf::from(parent)

    }

    fn split_path_prefix(last_arg: &String) -> (String, String) {
        let split  = last_arg.rsplit_once("/");
        match split {
            Some(("", filename)) => {
                ("/".to_owned(), filename.to_owned())
            },
            Some((dir, filename)) => {
                (dir.to_owned(), filename.to_owned())
            }
            _ => ("".to_owned(), last_arg.to_owned())
        }
    }

    fn get_completion(&self, parent: String, base: String, directory_to_search: PathBuf) -> Vec<String> {
        let list_of_files = (self.cur_dir_lister)(directory_to_search.to_str().unwrap());
        let temp_trie = Trie::new(list_of_files);
        let matches = temp_trie.prefix_search(base.as_ref());

        let mut full_path = PathBuf::from(&directory_to_search);
        let mut completed_path = PathBuf::from(&parent);
        if matches.len() == 1 {
            full_path.push(&matches[0]);
            completed_path.push(&matches[0]);

            let suffix = if full_path.is_dir() { "/" } else { " " };
            let completion = format!("{}{}", completed_path.display(), suffix);
            vec![completion]
        } else {
            let mut res = Vec::new();
            for matching in matches {
                let mut full_path_match = PathBuf::from(&directory_to_search);
                full_path_match.push(&matching);
                if full_path_match.is_dir() {  //TODO: Check if we should reduce calls?
                    res.push(format!("{}{}", matching, "/"))
                } else {
                    res.push(matching)
                }
            }
            res.sort();
            res
        }
    }
}

impl Completer for ReadLineHelper {

    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let input = &line[..pos];
        let tokens: Vec<Token> = tokenize(input);
        if tokens.len() == 1 && !input.ends_with(|c: char| c.is_whitespace()) { //command
            let mut matches: Vec<String> = self.command_trie.prefix_search(input);

            matches.sort();
            matches.dedup();

            if matches.len() == 1 {
                return Ok((0, vec![format!("{} ", &matches[0])]));
            }

            Ok((0, matches))
        } else if tokens.len() >= 1 && input.ends_with(|c: char| c.is_whitespace()) {
            match tokens.last() {
                Some(Str((x, _))) => {
                    let cur_dir = env::current_dir()?.to_owned();
                    let matches = self.get_completion("".to_owned(), "".to_owned(), cur_dir);

                    Ok((x.to_owned() + 1, matches))
                },
                _ => Ok((0, Vec::new()))
            }
        } else if tokens.len() >= 2 {
            match &tokens[..] {
                [_prefix @ .., Str((x, _)), Str((_, last_arg))] => {
                    let (parent, base) = Self::split_path_prefix(last_arg);
                    let directory_to_search = Self::get_dir_to_search(&parent);
                    let matches = self.get_completion(parent, base, directory_to_search);
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
