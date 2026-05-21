use std::{iter::Peekable, mem::take, str::Chars};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub kind: CommandKind,
    pub redirect_stdout: Option<(String, bool)>,
    pub redirect_stderr: Option<(String, bool)>,
}

#[derive(Debug, PartialEq)]
pub enum CommandKind {
    Empty,
    Unknown(String),
    InvalidArgs(String),
    Exit,
    Pwd,
    Cd(String),
    Echo(String),
    Type(String),
    External { program: String, args: Vec<String> },
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String), // regular word or quoted string
    RedirectStdout,
    RedirectStderr,
    RedirectAppendStdout,
    RedirectAppendStderr,
}

impl Token {
    fn is_redirect(&self) -> bool {
        matches!(
            self,
            Token::RedirectStdout
                | Token::RedirectStderr
                | Token::RedirectAppendStdout
                | Token::RedirectAppendStderr
        )
    }
}

fn redirect_token(fd: char, append: bool) -> Token {
    match (fd, append) {
        ('1', false) => Token::RedirectStdout,
        ('1', true) => Token::RedirectAppendStdout,
        (_, false) => Token::RedirectStderr,
        (_, true) => Token::RedirectAppendStderr,
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.trim().chars().peekable();
    let mut word = String::new();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' => {
                chars.next(); // skip space
                finish_word(&mut tokens, &mut word);
            }
            '\'' => {
                chars.next(); // skip opening '
                read_until(&mut chars, &mut word, '\'', false);
            }
            '"' => {
                chars.next(); // skip opening "
                read_until(&mut chars, &mut word, '"', true);
            }
            '\\' => {
                chars.next();
                if let Some(&esc) = chars.peek() {
                    chars.next();
                    word.push(esc);
                }
            }
            // file descriptor if we're not currently building a word
            '1'..='2' if word.is_empty() => {
                let fd = chars.next().unwrap();

                if (chars.next_if_eq(&'>')).is_some() {
                    let append = (chars.next_if_eq(&'>')).is_some();
                    tokens.push(redirect_token(fd, append));
                } else {
                    chars.next();
                    word.push(c);
                }
            }
            '>' if word.is_empty() => {
                chars.next();
                let append = (chars.next_if_eq(&'>')).is_some();
                tokens.push(redirect_token('1', append));
            }
            _ => {
                chars.next();
                word.push(c);
            }
        }
    }
    finish_word(&mut tokens, &mut word);
    tokens
}

pub fn parse(input: &str) -> Command {
    let tokens = tokenize(input);
    debug!(&tokens);

    if tokens.is_empty() {
        return Command {
            kind: CommandKind::Empty,
            redirect_stdout: None,
            redirect_stderr: None,
        };
    }

    let (words, redirect_stdout, redirect_stderr) = split_tokens(&tokens);

    let cmd_name = match words.first() {
        Some(name) => name.as_str(),
        None => {
            return Command {
                kind: CommandKind::Empty,
                redirect_stdout: None,
                redirect_stderr: None,
            };
        }
    };

    let kind = match cmd_name {
        "exit" => CommandKind::Exit,
        "pwd" => CommandKind::Pwd,
        "echo" => CommandKind::Echo(words[1..].join(" ")),
        "cd" => match words.get(1) {
            Some(path) => CommandKind::Cd(path.clone()),
            None => CommandKind::InvalidArgs(input.to_string()),
        },
        "type" => match words.get(1) {
            Some(name) => CommandKind::Type(name.clone()),
            None => CommandKind::InvalidArgs(input.to_string()),
        },
        _ => CommandKind::External {
            program: cmd_name.to_string(),
            args: words[1..].to_vec(),
        },
    };

    Command {
        kind,
        redirect_stdout,
        redirect_stderr,
    }
}

fn split_tokens(tokens: &[Token]) -> (Vec<String>, Option<(String, bool)>, Option<(String, bool)>) {
    let mut words = Vec::new();
    let mut redirect_stdout = None;
    let mut redirect_stderr = None;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(w) => {
                words.push(w.clone());
            }
            Token::RedirectStdout | Token::RedirectAppendStdout => {
                let append = matches!(tokens[i], Token::RedirectAppendStdout);
                if let Some(Token::Word(file)) = tokens.get(i + 1) {
                    redirect_stdout = Some((file.clone(), append));
                    i += 1; // skip filename
                }
            }
            Token::RedirectStderr | Token::RedirectAppendStderr => {
                let append = matches!(tokens[i], Token::RedirectAppendStderr);
                if let Some(Token::Word(file)) = tokens.get(i + 1) {
                    redirect_stderr = Some((file.clone(), append));
                    i += 1;
                }
            }
        }
        i += 1;
    }

    (words, redirect_stdout, redirect_stderr)
}

fn finish_word(tokens: &mut Vec<Token>, word: &mut String) {
    if !word.is_empty() {
        tokens.push(Token::Word(take(word)));
    }
}

fn read_until(chars: &mut Peekable<Chars>, word: &mut String, end: char, allow_escapes: bool) {
    while let Some(&ch) = chars.peek() {
        chars.next();
        if ch == end {
            break;
        }
        if allow_escapes && ch == '\\' {
            if let Some(&esc) = chars.peek() {
                chars.next();
                word.push(esc);
            }
        } else {
            word.push(ch);
        }
    }
}

#[cfg(test)]
mod tests;
