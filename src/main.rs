mod input_parser;
mod models;

use models::ShellCmd;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::stdout;

use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use rustyline::DefaultEditor;

const PROMPT: &'static str = "$ ";
const TILDE: &'static str = "~";

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

fn get_builtin(cmd: &String) -> Option<Builtin> {
    match cmd.as_str() {
        "exit" => Some(Builtin::Exit),
        "echo" => Some(Builtin::Echo),
        "type" => Some(Builtin::Type),
        "pwd" => Some(Builtin::Pwd),
        "cd" => Some(Builtin::Cd),
        _ => None,
    }
}

fn main() -> rustyline::Result<()>{
    let mut rl = DefaultEditor::new()?;

    loop {
        let rl_input = rl.readline(PROMPT);
        let input = match rl_input {
            Ok(line) => { line }
            Err(r) => { return Err(r) }
        };

        let cmd_res = get_cmd_args(input.as_str());

        if cmd_res.is_err() {
            eprintln!("{}", cmd_res.err().unwrap());
            continue;
        }

        let cmd = cmd_res.unwrap();

        let output_file = match get_file(cmd.redirection_path.as_ref(), cmd.redirection_append) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("shell: {}: {}: ", &cmd.redirection_path.unwrap_or("".to_string()), &e);
                continue
            },
        };

        let error_file = match get_file(cmd.err_redirection_path.as_ref(), cmd.err_redirection_append) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("shell: {}: {}: ", &cmd.redirection_path.unwrap_or("".to_string()), &e);
                continue
            },
        };


        execute(&cmd.args, output_file, error_file,);
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

fn display_prompt() {
    print!("{}", PROMPT);
    stdout().flush().unwrap();
}

fn get_cmd_args(input: &str) -> Result<ShellCmd, String> {
    input_parser::parse(input)
}

fn execute(args: &Vec<String>,
           out_file: Option<File>,
           err_file: Option<File>) {
    if args.is_empty() {
        return;
    }
    let builtin_opt = get_builtin(&args[0]);

    match builtin_opt {
        Some(x) => {
            let mut output = get_write(out_file);
            let mut err_out = get_write(err_file);
            match x {
                Builtin::Exit => execute_exit(0),
                Builtin::Echo => execute_echo(args, &mut output),
                Builtin::Type => execute_type(args, &mut output, &mut err_out),
                Builtin::Pwd => execute_pwd(&mut output),
                Builtin::Cd => execute_cd(args, &mut err_out),
            }
        }
        None => match get_cmd_path(&args[0]) {
            Some(_) => execute_command(&args, out_file, err_file),
            None => eprintln!("{}: command not found", args[0]),
        },
    }
}

fn get_write(file: Option<File>) -> Box<dyn Write> {
    match file {
        None => Box::new(stdout()),
        Some(f) => Box::new(f),
    }
}

fn execute_cd(args: &Vec<String>, err_out: &mut Box<dyn Write>) {
    if args.len() > 2 {
        write_out_ln(err_out, "cd: too many arguments");
        return;
    }

    let path = args.get(1).map(String::as_str).unwrap_or("~");

    let tilde_replaced_path_res = tilde_replaced_path(path);
    if tilde_replaced_path_res.is_none() {
        write_out_ln(err_out, "No home directory set");
        return;
    }

    let path = tilde_replaced_path_res.unwrap();
    match path.as_path().canonicalize() {
        Ok(path_buf) => {
            let true_path = path_buf.as_path();
            if !true_path.exists() {
                write_out_ln(err_out, &format!("{}: {}: No such file or directory", args[0], args[1]))
            } else if !true_path.is_dir() {
                write_out_ln(err_out, &format!("{}: {}: Not a directory", args[0], args[1]))
            } else {
                let cd_result = env::set_current_dir(true_path);
                match cd_result {
                    Ok(_) => {}
                    Err(_) => write_out_ln(err_out, &format!("{}: {}: No such file or directory", args[0], args[1])),
                }
            }
        }
        Err(_) => write_out_ln(err_out, &format!("cd: {}: No such file or directory", args[1])),
    }
}

fn tilde_replaced_path(path_str: &str) -> Option<PathBuf> {
    if path_str.contains(TILDE) {
        return match env::home_dir() {
            None => None,
            Some(dir) => {
                let paths = path_str.replace(TILDE, into_path_str(dir).as_str());
                Some(Path::new(paths.as_str()).to_path_buf())
            }
        };
    }
    Some(Path::new(path_str).to_path_buf())
}

fn execute_exit(code: i32) {
    exit(code)
}

fn execute_echo(args: &Vec<String>, output: &mut dyn Write) {
    let n = args.len();
    if n == 1 {
        write_out_ln(output, "");
        return;
    }

    for i in 1..(n - 1) {
        write_out(output, &format!("{} ", args[i]))
    }
    write_out_ln(output, &format!("{}", args[n - 1]))
}

fn write_out(output: &mut dyn Write, text: &str) {
    if let Err(e) = write!(output, "{}", text) {
        eprintln!("Error writing output: {}", e);
    }
}

fn write_out_ln(output: &mut dyn Write, text: &str) {
    if let Err(e) = writeln!(output, "{}", text) {
        eprintln!("Error writing output: {}", e);
    }
}

fn execute_type(args: &Vec<String>, output: &mut dyn Write, err_out: &mut dyn Write) {
    if args.len() < 2 {
        return;
    }

    let builtin_opt = get_builtin(&args[1]);
    match builtin_opt {
        Some(_) => write_out_ln(output, &format!("{} is a shell builtin", &args[1])),
        _ => match get_cmd_path(&args[1]) {
            Some(full_path) => write_out_ln(
                output,
                &format!("{} is {}", &args[1], into_path_str(full_path)),
            ),
            None => write_out_ln(err_out, &format!("{}: not found", &args[1])),
        },
    }
}

fn execute_pwd(output: &mut dyn Write) {
    let res = env::current_dir();
    match res {
        Ok(path) => write_out_ln(output, &format!("{}", &into_path_str(path))),
        Err(_) => {}
    }
}

fn execute_command(args: &Vec<String>, out_file: Option<File>, err_file: Option<File>) {
    let mut child_cmd = Command::new(&args[0]);
    child_cmd.args(&args[1..]);

    match out_file {
        None => {}
        Some(f) => {
            child_cmd.stdout(f);
        }
    };
    match err_file {
        None => {}
        Some(f) => {
            child_cmd.stderr(f);
        }
    };

    let child_res = child_cmd.spawn();

    match child_res {
        Ok(mut child) => match child.wait() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("failed wait on child: {}", e);
            }
        },
        Err(e) => {
            eprintln!("shell: failed to execute {}: {}", &args[0], e)
        }
    }
}

fn into_path_str(full_path: PathBuf) -> String {
    full_path.into_os_string().into_string().unwrap()
}

fn get_cmd_path(cmd: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for dir in env::split_paths(&paths) {
        let full_path = dir.as_path().join(cmd);
        if is_executable(&full_path) {
            return Some(full_path);
        }
    }
    None
}

fn is_executable(path_buf: &PathBuf) -> bool {
    match path_buf.metadata() {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}
