use crate::path;
use std::path::PathBuf;

pub const BUILTINS: [&str; 4] = ["type", "pwd", "echo", "exit"];

pub fn handle_type(name: &str, path: &[PathBuf]) {
    if BUILTINS.contains(&name) {
        println!("{} is a shell builtin", name);
    } else if let Some(found) = path::find_executable(name, path) {
        println!("{} is {}", name, found.display());
    } else {
        println!("{}: not found", name);
    }
}

pub fn execute_command(program: &str, args: Vec<&str>) {
    let _ = std::process::Command::new(program)
        .args(args)
        .spawn()
        .expect("Failed to execute command")
        .wait();
}
