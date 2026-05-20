use crate::path;
use std::path::{Path, PathBuf};

pub struct Shell {
    executables: Vec<PathBuf>,
    current_dir: PathBuf,
}

pub enum Command {
    Empty,
    Unknown(String),
    Exit,
    Pwd,
    Cd(String),
    Echo(String),
    Type(String),
    External(String, Vec<String>),
}

impl Shell {
    const BUILTINS: [&str; 4] = ["type", "pwd", "echo", "exit"];

    pub fn new() -> Self {
        Shell {
            executables: path::list_executables(),
            current_dir: std::env::current_dir().unwrap(),
        }
    }

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
            [cmd, args] if self.find_executable(cmd).is_some() => Command::External(
                cmd.to_string(),
                args.split(' ').map(|s| s.to_string()).collect(),
            ),
            [] => Command::Empty,
            [name, ..] => Command::Unknown(name.to_string()),
        }
    }

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::Exit => std::process::exit(0),
            Command::Pwd => println!("{}", self.current_dir.display()),
            Command::Echo(args) => println!("{}", args),
            Command::Type(name) => self.handle_type(name.as_str()),
            Command::Cd(path) => {
                let p = Path::new(&path);
                if p.is_dir() {
                    self.current_dir = p.to_path_buf()
                } else {
                    println!("cd: {}: No such file or directory", path);
                }
            }
            Command::Empty => {}
            Command::Unknown(cmd) => println!("{}: command not found", cmd),
            Command::External(program, args) => {
                std::process::Command::new(program)
                    .args(args)
                    .spawn()
                    .expect("Failed to execute command")
                    .wait()
                    .expect("failed to wait for command");
            }
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
