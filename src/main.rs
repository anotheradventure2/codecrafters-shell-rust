use codecrafters_shell::shell::Shell;
use std::io::{self, Write};

fn main() {
    let mut shell = Shell::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        shell.handle_input(&input);
    }
}
