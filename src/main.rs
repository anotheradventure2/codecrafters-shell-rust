use std::io::{self, Write};

fn main() {
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
                    } else {
                        println!("{}: not found", v[1])
                    }
                }
                _ => println!("{}: command not found", command.trim()),
            }
        }

        // if command == "exit" {
        //     break;
        // } else if is_builtin(command) {
        //     println!("{}", )
        // } else if command.starts_with("echo ") {
        //     let (_, args) = command.split_once(" ").unwrap();
        //     println!("{}", args)
        // } else {
        //     println!("{}: command not found", command.trim());
        // }
    }
}

// fn is_builtin(command: &str) -> bool {
//     let foo = ["type", "echo", "exit"]
//     return ["type", "echo", "exit"].iter().any(|&x| x == command.to_string().trim())
//     return ["type", "echo", "exit"].contains(&command)
// }
