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
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();

        match parts.as_slice() {
            ["exit"] => Command::Exit,
            ["pwd"] => Command::Pwd,
            ["echo", args] => Command::Echo(args.to_string()),
            ["cd", path] => Command::Cd(path.to_string()),
            ["type", name] => Command::Type(name.to_string()),
            [cmd] => {
                if let Some(full_path) = self.find_executable(cmd) {
                    Command::External(cmd.to_string(), Vec::new())
                } else {
                    Command::Unknown(cmd.to_string())
                }
            }
            [cmd, args] => {
                if let Some(full_path) = self.find_executable(cmd) {
                    Command::External(
                        cmd.to_string(),
                        args.split(' ').map(|s| s.to_string()).collect(),
                    )
                } else {
                    Command::Unknown(cmd.to_string())
                }
            }
            [] => Command::Empty,
            [name, ..] => Command::Unknown(name.to_string()),
        }
    }

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::Exit => std::process::exit(0),
            Command::Pwd => println!("{}", env::current_dir().unwrap().display()),
            Command::Echo(args) => println!("{}", args),
            Command::Type(name) => self.handle_type(name.as_str()),
            Command::Cd(path) => {
                let target_path = if path.starts_with('/') || path.starts_with('~') {
                    Path::new(&path).to_path_buf()
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
