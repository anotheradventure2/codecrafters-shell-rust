use codecrafters_shell::builtins;
use codecrafters_shell::path;
use std::env::current_dir;
use std::io::{self, Write};

fn main() {
    let executables = path::list_executables();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // split into [cmd, rest]
        let parts: Vec<&str> = input.trim().splitn(2, " ").collect();

        match parts.as_slice() {
            ["exit"] => break,
            ["pwd"] => println!("{}", current_dir().unwrap().display()),
            ["echo", args] => println!("{}", args),
            ["type", name] => builtins::handle_type(name, &executables),
            [cmd, args] if path::find_executable(cmd, &executables).is_some() => {
                builtins::execute_command(cmd, args.split(' ').collect())
            }
            _ => println!("{}: command not found", parts[0]),
        }
    }
}
