#[allow(unused_imports)]
use std::io::{self, Write};

const PROMPT: &'static str = "$ ";

fn main() {

    loop {
        display_prompt();
        let command = read_command();
        if (command == "exit") {
            break
        }
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
    cmd.as_str().trim().to_string()
}

fn execute(cmd: String) {
    println!("{}: command not found", cmd);
}
