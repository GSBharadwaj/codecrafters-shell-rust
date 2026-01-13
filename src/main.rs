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

fn main() {
    loop {
        display_prompt();
        let input = read_input();
        let cmd_res = get_cmd_args(input.as_str());

        if cmd_res.is_err() {
            eprintln!("{}", cmd_res.err().unwrap());
            continue;
        }

        let cmd = cmd_res.unwrap();

        let output_file = match get_file(&cmd) {
            Ok(value) => value,
            Err(_) => continue,
        };

        execute(&cmd.args, output_file);
    }
}

fn get_file(cmd: &ShellCmd) -> Result<Option<File>, io::Error> {
    match &cmd.redirection_path {
        None => Ok(None),
        Some(out_path) => {
            let fie_res = OpenOptions::new().create(true).write(true).truncate(true).append(false).open(&out_path);
            match fie_res {
                Err(e) => {
                    eprintln!("shell: {}: {}: ", &out_path, &e);
                    Err(e)
                }
                Ok(file) => Ok(Some(file)),
            }
        }
    }
}

fn display_prompt() {
    print!("{}", PROMPT);
    io::stdout().flush().unwrap();
}

fn read_input() -> String {
    let mut cmd = String::new();
    io::stdin().read_line(&mut cmd).unwrap();

    cmd
}

fn get_cmd_args(input: &str) -> Result<ShellCmd, String> {
    input_parser::parse(input)
}

fn execute(args: &Vec<String>, out_file: Option<File>) {
    if args.is_empty() {
        return;
    }
    let builtin_opt = get_builtin(&args[0]);

    match builtin_opt {
        Some(x) => {
            let mut output = get_write(out_file);
            match x {
                Builtin::Exit => execute_exit(0),
                Builtin::Echo => execute_echo(args, &mut output),
                Builtin::Type => execute_type(args, &mut output),
                Builtin::Pwd => execute_pwd(&mut output),
                Builtin::Cd => execute_cd(args),
            }
        }
        None => match get_cmd_path(&args[0]) {
            Some(_) => execute_command(&args, out_file),
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

fn execute_cd(args: &Vec<String>) {
    if args.len() != 2 {
        eprintln!("Usage: cd <directory>");
        return;
    }

    let tilde_replaced_path_res = tilde_replaced_path(&args[1]);
    if tilde_replaced_path_res.is_none() {
        eprintln!("No home directory set");
        return;
    }

    let path = tilde_replaced_path_res.unwrap();
    match path.as_path().canonicalize() {
        Ok(path_buf) => {
            let true_path = path_buf.as_path();
            if !true_path.exists() {
                eprintln!("{}: {}: No such file or directory", args[0], args[1])
            } else if !true_path.is_dir() {
                eprintln!("{}: {}: Not a directory", args[0], args[1])
            } else {
                let cd_result = env::set_current_dir(true_path);
                match cd_result {
                    Ok(_) => {}
                    Err(_) => eprintln!("{}: {}: No such file or director", args[0], args[1]),
                }
            }
        }
        Err(_) => eprintln!("cd: {}: No such file or directory", args[1]),
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
    if n < 2 {
        eprintln!("Need at least one argument");
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

fn execute_type(args: &Vec<String>, output: &mut dyn Write) {
    if args.len() < 2 {
        eprintln!("Need at least one argument");
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
            None => eprintln!("{}: not found", &args[1]),
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

fn execute_command(args: &Vec<String>, file: Option<File>) {
    let mut child_cmd = Command::new(&args[0]);
    child_cmd.args(&args[1..]);

    match file {
        None => {}
        Some(f) => {
            child_cmd.stdout(f);
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
