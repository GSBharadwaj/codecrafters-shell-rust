use std::alloc::System;
use std::collections::{HashSet};

#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

const PROMPT: &'static str = "$ ";


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

    let mut builtins:HashSet<&str> = HashSet::new();

    builtins.insert("echo");
    builtins.insert("exit");
    builtins.insert("type");

    match args[0] {
        "echo" =>  execute_echo(args),
        "type" =>  execute_type(builtins, args),
        "exit" => execute_exit(0),
        _ => print!("{}: command not found", args[0])
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

fn execute_type(builtins: HashSet<&str>, args: Vec<&str>) {
    if args.len() < 2 {
        panic!("Need at least one argument")
    }
    if builtins.contains(args[1]) {
        print!("{} is a shell builtin", &args[1])
    } else {
        print!("{}: not found", &args[1])
    }
}
