#[allow(unused_imports)]
use std::io::{self, Write};

const PROMPT: &'static str = "$ ";

fn main() {

    print!("{}", PROMPT);
    io::stdout().flush().unwrap();

    let command = read_command();
    execute(command);
}

fn read_command() -> String {
    let mut cmd = String::new();
    io::stdin().read_line(&mut cmd).unwrap();
    cmd
}

fn execute(cmd: String) {
    println!("{}: command not found", cmd.as_str().trim());
}
