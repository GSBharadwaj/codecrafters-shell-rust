mod input_parser;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::{exit, Command};
use std::{env};
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

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
        _ => None
    }
}

fn main() {

    loop {
        display_prompt();
        let input = read_input();
        let args = get_cmd_args(&input);
        execute(&args);
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

fn get_cmd_args(input: &String) -> Vec<String> {
    input_parser::parse(input)
}

fn execute(args: &Vec<String>) {
    if args.is_empty() {return;}
    let builtin_opt = get_builtin(&args[0]);

    match builtin_opt {
        Some(Builtin::Exit) => execute_exit(0),
        Some(Builtin::Echo) => execute_echo(args),
        Some(Builtin::Type) => execute_type(args),
        Some(Builtin::Pwd) => execute_pwd(),
        Some(Builtin::Cd) => execute_cd(args),
        None => match get_cmd_path(&args[0]) {
            Some(_) => execute_command(&args),
            None => println!("{}: command not found", args[0])
        }
    }
}

fn execute_cd(args: &Vec<String>) {
    if args.len() != 2 {
        println!("Usage: cd <directory>");
        return;
    }

    let tilde_replaced_path_res = tilde_replaced_path(&args[1]);
    if tilde_replaced_path_res.is_none() {
        println!("No home directory set");
        return;
    }

    let path = tilde_replaced_path_res.unwrap();
    match path.as_path().canonicalize() {
        Ok(path_buf) => {
            let true_path = path_buf.as_path();
            if !true_path.exists() {
                println!("{}: {}: No such file or directory", args[0], args[1])
            } else if !true_path.is_dir() {
                println!("{}: {}: Not a directory", args[0], args[1])
            } else {
                let cd_result = env::set_current_dir(true_path);
                match cd_result {
                    Ok(_) => {}
                    Err(_) => println!("{}: {}: No such file or director", args[0], args[1])
                }
            }
        }
        Err(_) => println!("cd: {}: No such file or directory", args[1])
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
        }
    }
    Some(Path::new(path_str).to_path_buf())
}

fn execute_exit(code: i32) {
    exit(code)
}

fn execute_echo(args: &Vec<String>) {
    let n = args.len();
    if n < 2 {
        println!("Need at least one argument");
        return;
    }

    for i in 1..(n - 1) {
        print!("{} ", args[i])
    }
    println!("{}", args[n - 1])
}

fn execute_type(args: &Vec<String>) {
    if args.len() < 2 {
        println!("Need at least one argument");
        return;
    }

    let builtin_opt = get_builtin(&args[1]);
    match builtin_opt {
        Some(_) => println!("{} is a shell builtin", &args[1]),
        _ => match get_cmd_path(&args[1]) {
            Some(full_path) =>  println!("{} is {}", &args[1], into_path_str(full_path)),
            None => println!("{}: not found", &args[1]),
        },
    }
}

fn execute_pwd() {
    let res = env::current_dir();
    match res {
        Ok(path) => {println!("{}", &into_path_str(path))}
        Err(_) => {}
    }
}

fn execute_command(args: &Vec<String>) {
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .spawn()
        .expect("failed to execute child process");

    child.wait().expect("failed wait on child");
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
    match path_buf.metadata() {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false
    }
}
