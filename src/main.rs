#[allow(unused_imports)]
use std::io::{self, Write};

const PROMPT: &'static str = "$ ";

fn main() {

    loop {
        display_prompt();
        let command = read_command();
        execute(command);
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

fn execute(cmd: String) {
    println!("{}: command not found", cmd.as_str().trim());
}
