use crate::path;
use std::{
    env,
    path::{Path, PathBuf},
};

pub struct Shell {
    executables: Vec<PathBuf>,
}

enum Command {
    Empty,
    Unknown(String),
    Exit,
    Pwd,
    Cd(String),
    Echo(String),
    Type(String),
    External(String, Vec<String>),
}

impl Default for Shell {
    fn default() -> Self {
        Shell {
            executables: path::list_executables(),
        }
    }
}

impl Shell {
    const BUILTINS: [&str; 5] = ["type", "pwd", "echo", "exit", "cd"];

    pub fn handle_input(&mut self, input: &str) {
        let command = self.parse(input.trim());
        self.execute(command);
    }

    fn parse(&self, input: &str) -> Command {
        let parts: Vec<String> = self.parse_input(&input);
        let parts: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

        match parts.as_slice() {
            ["exit"] => Command::Exit,
            ["pwd"] => Command::Pwd,
            ["echo", args @ ..] => Command::Echo(args.join(" ")),
            ["cd", path] => Command::Cd(path.to_string()),
            ["type", name] => Command::Type(name.to_string()),
            [cmd] if self.find_executable(cmd).is_some() => {
                Command::External(cmd.to_string(), Vec::new())
            }
            [cmd, args @ ..] if self.find_executable(cmd).is_some() => Command::External(
                cmd.to_string(),
                args.iter().map(|e| e.to_string()).collect(),
            ),
            [] => Command::Empty,
            [name, ..] => Command::Unknown(name.to_string()),
        }
    }

    fn parse_input(&self, input: &str) -> Vec<String> {
        let mut in_double_quotes = false;
        let mut in_quotes = false;
        let mut current_arg = String::new();
        let mut args = Vec::new();
        let mut prev_was_escape = false;

        for c in input.chars() {
            if prev_was_escape {
                let escaped = match c {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    _ => c,
                };
                current_arg.push(escaped);
                prev_was_escape = false;
            } else if c == '\\' {
                prev_was_escape = true;
            } else if c == '"' {
                in_double_quotes = !in_double_quotes;
            } else if c == '\'' {
                in_quotes = !in_quotes
            } else if c == ' ' && !in_quotes && !in_double_quotes {
                if !current_arg.is_empty() {
                    args.push(current_arg);
                    current_arg = String::new();
                }
            } else {
                current_arg.push(c);
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::Exit => std::process::exit(0),
            Command::Pwd => println!("{}", env::current_dir().unwrap().display()),
            Command::Echo(args) => println!("{}", args),
            Command::Type(name) => self.handle_type(name.as_str()),
            Command::Cd(path) => {
                let target_path = if path.starts_with('/') {
                    Path::new(&path).to_path_buf()
                } else if path.starts_with('~') {
                    let a = env::var("HOME").unwrap();
                    Path::new(&a).to_path_buf()
                } else {
                    env::current_dir().unwrap().join(path)
                };

                if target_path.is_dir() {
                    env::set_current_dir(target_path.as_path()).expect("failed to set path");
                } else {
                    println!("cd: {}: No such file or directory", target_path.display());
                }
            }
            Command::External(program, args) => {
                std::process::Command::new(program)
                    .args(args)
                    .spawn()
                    .expect("Failed to execute command")
                    .wait()
                    .expect("failed to wait for command");
            }
            Command::Empty => {}
            Command::Unknown(cmd) => println!("{}: command not found", cmd),
        }
    }

    fn find_executable(&self, name: &str) -> Option<&PathBuf> {
        self.executables
            .iter()
            .find(|p| p.file_name().map(|f| f == name).unwrap_or(false))
    }

    fn handle_type(&self, name: &str) {
        if Shell::BUILTINS.contains(&name) {
            println!("{} is a shell builtin", name);
        } else if let Some(found) = self.find_executable(name) {
            println!("{} is {}", name, found.display());
        } else {
            println!("{}: not found", name);
        }
    }
}
