use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Context;

pub struct ReadLineHelper {
    commands: Vec<String>
}

impl ReadLineHelper {
    pub fn set_commands(&mut self, commands: Vec<String>) {
        self.commands = commands
    }
}

impl Default for ReadLineHelper {
    fn default() -> Self {
        Self {
            commands : vec![]
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

        let matches: Vec<String> = self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .cloned().collect();
        Ok((0, matches))
    }
}

impl Hinter for ReadLineHelper { type Hint = String; }
impl Highlighter for ReadLineHelper {}
impl Validator for ReadLineHelper {}
impl rustyline::Helper for ReadLineHelper {}
