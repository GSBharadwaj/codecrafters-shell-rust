mod input_parser;
mod models;
mod readline_helper;
mod trie;
mod builtin;
mod util;

use crate::builtin::builtin::{execute_builtin, Builtin};
use crate::readline_helper::ReadLineHelper;
use crate::util::util::{get_all_executables, get_cmd_path};
use models::ShellCmd;
use rustyline::Editor;
use rustyline::{CompletionType, Config};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self};
use std::io::{pipe, PipeReader, PipeWriter};
use std::process::{Child, Command};
use std::str::FromStr;
use rustyline::history::DefaultHistory;

const PROMPT: &'static str = "$ ";

fn main() -> rustyline::Result<()>{
    let builtin_commands = vec![
                                      "echo".to_string(),
                                      "exit".to_string(),
                                      "type".to_string(),
                                      "pwd".to_string(),];
    let mut all_commands = get_all_executables();
    all_commands.extend(builtin_commands);

    let readline_helper = ReadLineHelper::new(all_commands);

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .history_ignore_space(false)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(readline_helper));


    loop {
        let rl_input = rl.readline(PROMPT);
        let input = match rl_input {
            Ok(line) => { line }
            Err(r) => { return Err(r) }
        };

        let cmd_res = get_cmd_args(input.as_str());
        let _ = rl.add_history_entry(input);

        if cmd_res.is_err() {
            eprintln!("{}", cmd_res.err().unwrap());
            continue;
        }

        let cmd_list = cmd_res.unwrap();
        if cmd_list.len() < 1 {
            continue
        }

        let mut child_processes = Vec::new();
        let mut pipes = VecDeque::new();
        for (i, cmd) in cmd_list.iter().enumerate() {
            let next = cmd_list.get(i + 1);
            let reader =  pipes.pop_back();

            let writer =
                if let Some((reader, writer)) = next.map(|_| pipe().ok()).flatten() {
                    pipes.push_front(reader);
                    Some(writer)
                } else if next.is_some() {
                    eprintln!("shell: {}: failed to create pipe", cmd.args[0]);
                    continue;
                } else {
                    None
                };

            let output_file = match get_file(cmd.redirection_path.as_ref(), cmd.redirection_append) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("shell: {}: {}: ", cmd.redirection_path.as_deref().unwrap_or(""), &e);
                    continue
                },
            };

            let error_file = match get_file(cmd.err_redirection_path.as_ref(), cmd.err_redirection_append) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("shell: {}: {}: ", &cmd.err_redirection_path.as_deref().unwrap_or(""), &e);
                    continue
                },
            };

            if let Some(child) = execute(&cmd.args, output_file, error_file, &rl, reader, writer) {
                child_processes.push((i, child))
            }
        }

        for (i, child_process) in child_processes {
            let outcome = child_process.and_then(|mut child| child.wait());
            if let Err(e) = outcome {
                let cmd_name = &cmd_list[i].args[0];
                eprintln!("shell: failed to execute {cmd_name}: {e}")
            }
        }
    }
}

fn get_file(file_path: Option<&String>, should_append: bool) -> Result<Option<File>, io::Error> {
    match file_path {
        None => Ok(None),
        Some(out_path) => {
            let fie_res =
                OpenOptions::new()
                    .create(true) .write(true)
                    .truncate(!should_append)
                    .append(should_append)
                    .open(&out_path);

            match fie_res {
                Err(e) => {
                    Err(e)
                }
                Ok(file) => Ok(Some(file)),
            }
        }
    }
}


fn get_cmd_args(input: &str) -> Result<Vec<ShellCmd>, String> {
    input_parser::parse(input)
}

fn execute(args: &Vec<String>,
           out_file: Option<File>,
           err_file: Option<File>,
           rl: &Editor<ReadLineHelper, DefaultHistory>,
           reader: Option<PipeReader>,
           writer: Option<PipeWriter> ) -> Option<io::Result<Child>> {
    if args.is_empty() {
        return None;
    }

    let builtin_opt = Builtin::from_str(&args[0]);

    match builtin_opt {
        Ok(x) => {
            execute_builtin(x, args, out_file, err_file, rl, reader, writer);
            None
        }
        Err(()) => match get_cmd_path(&args[0]) {
            Some(_) => {spawn_command(&args, out_file, err_file, reader, writer)},
            None => {eprintln!("{}: command not found", args[0]); None}
        },
    }
}

fn spawn_command(args: &Vec<String>,
                 out_file: Option<File>,
                 err_file: Option<File>,
                 reader: Option<PipeReader>,
                 writer: Option<PipeWriter>) -> Option<io::Result<Child>> {
    let mut child_cmd = Command::new(&args[0]);
    child_cmd.args(&args[1..]);

    match out_file {
        None => {
            if let Some(f) = writer  {
                child_cmd.stdout(f);
            }
        }
        Some(f) => {
            child_cmd.stdout(f);
        }
    };
    match reader {
        None => {}
        Some(r) => {
            child_cmd.stdin(r);
        }
    }
    match err_file {
        None => {}
        Some(f) => {
            child_cmd.stderr(f);
        }
    };

    Some(child_cmd.spawn())
}
