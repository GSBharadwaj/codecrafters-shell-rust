#[allow(unused_imports)]
use std::io::{self, Write};

const PROMPT: &'static str = "$ ";

fn main() {

    loop {
        display_prompt();
        let command = read_command();
        let args = get_cmd_args(&command);
        if args[0] == "exit" {
            break
        }

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
    match args[0] {
        "echo" =>  execute_echo(args),
        _ => print!("{}: command not found", args[0])
    }
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
