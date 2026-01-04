
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use crate::Builtin::{Echo, Exit, Type, INVALID};

const PROMPT: &'static str = "$ ";

enum Builtin {
    Exit,
    Echo,
    Type,
    INVALID
}

fn get_builtin(cmd: &str) -> Builtin {
    match cmd {
        "exit" => Exit,
        "echo" => Echo,
        "type" => Type,
        _ => INVALID
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
        Exit => execute_exit(0),
        Echo =>  execute_echo(args),
        Type =>  execute_type(args),
        INVALID => print!("{}: command not found", args[0])
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

    let builtin = get_builtin(args[1]);
    match builtin {
        INVALID => print!("{}: not found", &args[1]),
        _ => print!("{} is a shell builtin", &args[1])
    }
}
