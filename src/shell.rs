use crate::parser;
use crate::parser::Command;
use crate::path;
use std::{
    env,
    path::{Path, PathBuf},
};

pub struct Shell {
    executables: Vec<PathBuf>,
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
        let command = parser::parse(&input);
        debug!(&command);

        self.execute(&command);
    }

    fn execute(&mut self, cmd: &Command) {
        match cmd {
            Command::Exit => std::process::exit(0),
            Command::Pwd => println!("{}", env::current_dir().unwrap().display()),
            Command::Echo(args) => println!("{}", args),
            Command::Type(name) => self.handle_type(name),
            Command::Cd(path) => {
                let target = self.resolve_path(&path);

                if target.is_dir() {
                    env::set_current_dir(&target).expect("failed to set path");
                } else {
                    println!("cd: {}: No such file or directory", target.display());
                }
            }
            Command::External { program, args, .. } => {
                std::process::Command::new(program)
                    .args(args)
                    .spawn()
                    .expect("Failed to execute command")
                    .wait()
                    .expect("failed to wait for command");
            }
            Command::Empty => {}
            Command::InvalidArgs(input) => println!("{} invalid arguments", input),
            Command::Unknown(cmd) => println!("{}: command not found", cmd),
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        if path.starts_with('/') {
            Path::new(path).to_path_buf()
        } else if path.starts_with('~') {
            Path::new(&env::var("HOME").unwrap()).to_path_buf()
        } else {
            env::current_dir().unwrap().join(path)
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
