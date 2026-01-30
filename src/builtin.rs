pub mod builtin {
    use crate::readline_helper::ReadLineHelper;
    use crate::util::util::{get_cmd_path, into_path_str, TILDE};
    use rustyline::history::{DefaultHistory, History, SearchDirection};
    use rustyline::Editor;
    use std::borrow::Cow;
    use std::env;
    use std::fs::File;
    use std::io::{stdout, PipeReader, PipeWriter, Write};
    use std::path::{Path, PathBuf};
    use std::process::exit;
    use std::str::FromStr;

    pub enum Builtin {
        Exit,
        Echo,
        Type,
        Pwd,
        Cd,
        History,
    }

    impl FromStr for Builtin {
        type Err = ();

        fn from_str(s: &str) -> Result<Builtin, ()> {
            match s {
                "exit" => Ok(Builtin::Exit),
                "echo" => Ok(Builtin::Echo),
                "type" => Ok(Builtin::Type),
                "pwd" => Ok(Builtin::Pwd),
                "cd" => Ok(Builtin::Cd),
                "history" => Ok(Builtin::History),
                _ => Err(()),
            }
        }
    }

    pub fn execute_builtin(builtin: Builtin,
                           args: &Vec<String>,
                           out_file: Option<File>,
                           err_file: Option<File>,
                           rl: &mut Editor<ReadLineHelper, DefaultHistory>,
                           _: Option<PipeReader>,
                           writer: Option<PipeWriter>) {
        let mut out = get_write(out_file, writer);
        let mut err_out = get_write(err_file, None);
        match builtin {
            Builtin::Exit => execute_exit(0),
            Builtin::Echo => execute_echo(args, &mut out),
            Builtin::Type => execute_type(args, &mut out, &mut err_out),
            Builtin::Pwd => execute_pwd(&mut out),
            Builtin::Cd => execute_cd(args, &mut err_out),
            Builtin::History => execute_history(args, rl, &mut out, &mut err_out),
        }
    }

    fn execute_history(args: &Vec<String>,
                       rl: &mut Editor<ReadLineHelper, DefaultHistory>,
                       out: &mut Box<dyn Write>,
                       err_out: &mut Box<dyn Write>) {
        if args.len() > 3 {
            write_out_ln(err_out, "shell: history: too many arguments");
            return;
        }


        if let Some(arg) = args.get(1) {
            match arg.as_str() {
                "-r" => {
                    if let Some(path) = args.get(2) {
                        let _ = rl.load_history(path);
                    }
                },
                _ => {
                    match arg.parse::<usize>() {
                        Ok(i) => { print_history(rl, out, i) }
                        Err(_) => {
                            write_out_ln(err_out, &format!("shell: history: {arg}: numeric argument required"));
                            return;
                        }
                    }
                }
            }
        } else {
            print_history(rl, out, rl.history().len())
        };

    }

    fn print_history(rl: &Editor<ReadLineHelper, DefaultHistory>, out: &mut Box<dyn Write>, limit: usize) {
        let history = rl.history();

        let mut i = history.len() - limit;
        let places = if history.len() > 1000 { 5 } else { 4 };

        while i < history.len() {
            if let Ok(search_res) = history.get(i, SearchDirection::Forward) {
                if let Some(res) = search_res {
                    let cmd = Cow::into_owned(res.entry);
                    let text = &format!("{:places$} {cmd}", i + 1, places = places);
                    write_out_ln(out, text);
                }
            }
            i += 1;
        }
    }

    fn execute_cd(args: &Vec<String>, err_out: &mut Box<dyn Write>) {
        if args.len() > 2 {
            write_out_ln(err_out, "cd: too many arguments");
            return;
        }

        let path = args.get(1).map(String::as_str).unwrap_or("~");

        let tilde_replaced_path_res = tilde_replaced_path(path);
        if tilde_replaced_path_res.is_none() {
            write_out_ln(err_out, "No home directory set");
            return;
        }

        let path = tilde_replaced_path_res.unwrap();
        match path.as_path().canonicalize() {
            Ok(path_buf) => {
                let true_path = path_buf.as_path();
                if !true_path.exists() {
                    write_out_ln(
                        err_out,
                        &format!("{}: {}: No such file or directory", args[0], args[1]),
                    )
                } else if !true_path.is_dir() {
                    write_out_ln(
                        err_out,
                        &format!("{}: {}: Not a directory", args[0], args[1]),
                    )
                } else {
                    let cd_result = env::set_current_dir(true_path);
                    match cd_result {
                        Ok(_) => {}
                        Err(_) => write_out_ln(
                            err_out,
                            &format!("{}: {}: No such file or directory", args[0], args[1]),
                        ),
                    }
                }
            }
            Err(_) => write_out_ln(
                err_out,
                &format!("cd: {}: No such file or directory", args[1]),
            ),
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
        if n == 1 {
            write_out_ln(output, "");
            return;
        }

        for i in 1..(n - 1) {
            write_out(output, &format!("{} ", args[i]))
        }
        write_out_ln(output, &format!("{}", args[n - 1]))
    }

    fn execute_type(args: &Vec<String>, output: &mut dyn Write, err_out: &mut dyn Write) {
        if args.len() < 2 {
            return;
        }

        let builtin_opt = Builtin::from_str(&args[1]);
        match builtin_opt {
            Ok(_) => write_out_ln(output, &format!("{} is a shell builtin", &args[1])),
            Err(_) => match get_cmd_path(&args[1]) {
                Some(full_path) => write_out_ln(
                    output,
                    &format!("{} is {}", &args[1], into_path_str(full_path)),
                ),
                None => write_out_ln(err_out, &format!("{}: not found", &args[1])),
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


    fn get_write(file: Option<File>, pipe: Option<PipeWriter>) -> Box<dyn Write> {
        match file {
            None => match pipe {
                None => Box::new(stdout()),
                Some(p) => Box::new(p),
            },
            Some(f) => Box::new(f),
        }
    }
}
