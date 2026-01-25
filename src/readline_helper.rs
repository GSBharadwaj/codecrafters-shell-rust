use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Context;
use crate::trie::trie::Trie;

pub struct ReadLineHelper {
    command_trie: Trie,
}

impl ReadLineHelper {
    pub fn new(commands: Vec<String>) -> Self {
        Self {
            command_trie: Trie::new(commands),
        }
    }
}

impl Completer for ReadLineHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let input = &line[..pos];
        if input.is_empty() {
            return Ok((0, Vec::new()))
        }

        let mut matches: Vec<String> = self.command_trie.prefix_search(input);

        matches.sort();
        matches.dedup();

        if matches.len() == 1 {
            return Ok((0, vec![format!("{} ", &matches[0])]))
        }

        Ok((0, matches))
    }

}

impl Hinter for ReadLineHelper { type Hint = String; }
impl Highlighter for ReadLineHelper {}
impl Validator for ReadLineHelper {}
impl rustyline::Helper for ReadLineHelper {}
