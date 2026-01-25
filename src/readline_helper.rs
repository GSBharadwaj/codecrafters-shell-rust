use rustyline::completion::{extract_word, Completer};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::validate::Validator;
use rustyline::{Changeset, Context};
use std::cell::Cell;

pub struct ReadLineHelper {
    commands: Vec<String>,
    unique_match_for_completion: Cell<Option<bool>>,
}

impl ReadLineHelper {
    pub fn set_commands(&mut self, commands: Vec<String>) {
        self.commands = commands
    }
}

impl Default for ReadLineHelper {
    fn default() -> Self {
        Self {
            commands : vec![],
            unique_match_for_completion: Cell::new(None),
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

        let (start, _) = extract_word(line, pos, None, |s|  s.is_whitespace());

        let mut matches: Vec<String> = self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .cloned().collect();
        matches.sort();
        matches.dedup();
        self.unique_match_for_completion.set(Some(matches.len() == 1));
        Ok((start, matches))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str, cl: &mut Changeset) {
        let completion =
            match self.unique_match_for_completion.get() {
                Some(true) => {
                    format!("{} ", elected)
                },
                _ => elected.to_string()
            };
        line.replace(start..line.pos(), &completion, cl);
    }
}

impl Hinter for ReadLineHelper { type Hint = String; }
impl Highlighter for ReadLineHelper {}
impl Validator for ReadLineHelper {}
impl rustyline::Helper for ReadLineHelper {}
