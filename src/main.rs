
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::{exit, Command};
use std::{env};
use std::path::{Path, PathBuf};

const PROMPT: &'static str = "$ ";

enum Builtin {
    Exit,
    Echo,
    Type,
}

fn get_builtin(cmd: &str) -> Option<Builtin> {
    match cmd {
        "exit" => Some(Builtin::Exit),
        "echo" => Some(Builtin::Echo),
        "type" => Some(Builtin::Type),
        _ => None
    }
}

fn main() {

    loop {
        display_prompt();
        let command = read_command();
        let args = get_cmd_args(&command);
        execute(args);
    }
}

fn display_prompt() {
    print!("{}", PROMPT);
    io::stdout().flush().unwrap();
}

fn read_command() -> String {
    let mut cmd = String::new();
    io::stdin().read_line(&mut cmd).unwrap();

    cmd
}

fn get_cmd_args(cmd: &String) -> Vec<&str> {
    cmd.as_str().split_whitespace()
        .into_iter()
        .map(str::trim)
        .collect()
}

fn execute(args: Vec<&str>) {
    let builtin_opt = get_builtin(args[0]);

    match builtin_opt {
        Some(Builtin::Exit) => execute_exit(0),
        Some(Builtin::Echo) => execute_echo(args),
        Some(Builtin::Type) => execute_type(args),
        None => match get_cmd_path(args[0]) {
            Some(_) => execute_command(&args),
            None => println!("{}: command not found", args[0])
        }
    }
}

fn execute_command(args: &Vec<&str>) {
    let mut cmd = Command::new(args[0]);
    for i in 1..args.len() {
        cmd.arg(args[i]);
    }
    let mut child = cmd.spawn().expect("failed to execute child process");
    child.wait().expect("failed wait on child");
}

fn execute_exit(code: i32) {
    exit(code)
}

fn execute_echo(args: Vec<&str>) {
    let n = args.len();
    if n < 2 {
        panic!("Need at least one argument")
    }

    for i in 1..(n - 1) {
        print!("{} ", args[i])
    }
    println!("{}", args[n - 1])
}

fn execute_type(args: Vec<&str>) {
    if args.len() < 2 {
        panic!("Need at least one argument")
    }

    let builtin_opt = get_builtin(args[1]);
    match builtin_opt {
        Some(_) => println!("{} is a shell builtin", &args[1]),
        _ => match get_cmd_path(args[1]) {
            Some(full_path) =>  println!("{} is {}", &args[1], into_path_str(full_path)),
            None => println!("{}: not found", &args[1]),
        },
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
            return Some(full_path)
        }
    }
    None
}

fn is_executable(path_buf: &PathBuf) -> bool {
    let metadata_res = path_buf.metadata();
    if !metadata_res.is_ok() {
        false
    } else {
        use std::os::unix::fs::PermissionsExt;

        let metadata = metadata_res.unwrap();
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
}
