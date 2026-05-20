use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{
    env,
    io::{self, Write},
    path::Path,
};

fn main() {
    let path: Vec<PathBuf> = env::var("PATH")
        .unwrap()
        .split(":")
        .flat_map(get_executables)
        .collect();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        if !command.contains(" ") {
            // commands with no args
            match command {
                "exit" => break,
                _ => println!("{}: command not found", command.trim()),
            }
        } else {
            // commands with args
            let v: Vec<&str> = command.split(" ").collect();

            match v[0] {
                "echo" => println!("{}", v[1..v.len()].join(" ")),
                "type" => {
                    if ["type", "echo", "exit"].contains(&v[1]) {
                        println!("{} is a shell builtin", v[1])
                    } else if let Some(found) = path
                        .iter()
                        .find(|e| e.file_name().is_some_and(|f| f == v[1]))
                    {
                        println!("{} is {}", v[1], found.to_string_lossy())
                    } else {
                        println!("{}: not found", v[1])
                    }
                }
                _ => println!("{}: command not found", command.trim()),
            }
        }
    }
}

fn get_executables(path: &str) -> Vec<PathBuf> {
    Path::new(path)
        .read_dir()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let full_path = e.path();
            if full_path.is_file()
                && full_path.metadata().unwrap().permissions().mode() & 0o111 != 0
            {
                Some(full_path)
            } else {
                None
            }
        })
        .collect()
}
