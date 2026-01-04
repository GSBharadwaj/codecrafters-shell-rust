
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use std::{env};
use std::path::{PathBuf};

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
        println!()
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
        None => print!("{}: command not found", args[0])
    }
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
    print!("{}", args[n - 1])
}

fn execute_type(args: Vec<&str>) {
    if args.len() < 2 {
        panic!("Need at least one argument")
    }

    let builtin_opt = get_builtin(args[1]);
    match builtin_opt {
        Some(_) => print!("{} is a shell builtin", &args[1]),
        _ => {
            let cmd_root = get_cmd_directory(args[1]);
            match cmd_root {
                None => print!("{}: not found", &args[1]),
                Some(full_path) => {
                    print!("{} is {}", &args[1], &(full_path.into_os_string().into_string().unwrap()))
                }
            }
        }
    }
}

fn get_cmd_directory(cmd: &str) -> Option<PathBuf> {
    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                if !path.as_path().is_dir() {
                    continue
                }

                let entries = std::fs::read_dir(path.as_path()).unwrap();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name();
                        if is_executable(entry.path()) && file_name == cmd {
                            return Some(entry.path());
                        }
                    }
                }
            }
            None
        }
        None => None
    }
}

#[cfg(unix)]
fn is_executable(path_buf: PathBuf) -> bool {
    let metadata_res = path_buf.as_path().metadata();
    if !metadata_res.is_ok() {
        false
    } else {
        use std::os::unix::fs::PermissionsExt;

        let metadata = metadata_res.unwrap();
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
}

#[cfg(windows)]
fn is_executable(path_buf: PathBuf) -> bool {
    match path_buf.as_path().extension() {
        None => false,
        Some(ex) => match ex.to_ascii_lowercase().into_string().unwrap().as_str()
        {
            "exe" | "cmd" | "bat" | "com" => { true }
            | _ => false
        }
    }
}