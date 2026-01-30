use std::iter::Skip;
use rustyline::Editor;
use rustyline::history::{DefaultHistory, History};
use crate::readline_helper::ReadLineHelper;

pub struct ShellCmd {
    pub args: Vec<String>,
    pub redirection_path: Option<String>,
    pub redirection_append: bool,
    pub err_redirection_path: Option<String>,
    pub err_redirection_append: bool,
}

pub struct ShellMetadata {
    pub rl: Editor<ReadLineHelper, DefaultHistory>,
    append_save_point: usize,
}


impl ShellMetadata {
    pub fn from(rl: Editor<ReadLineHelper, DefaultHistory>) -> Self {
        let append_save_point = rl.history().len();
        Self {
            rl,
            append_save_point
        }
    }

    pub fn update_save_point(&mut self) {
        self.append_save_point = self.rl.history().len();
    }

    pub fn iter_from_save_point(&mut self) -> Skip<impl DoubleEndedIterator<Item=&String>> {
        self.rl.history().iter().skip(self.append_save_point)
    }
}
