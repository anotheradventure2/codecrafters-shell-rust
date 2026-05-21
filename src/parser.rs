use std::{iter::Peekable, mem::take, str::Chars};

#[derive(Debug, PartialEq)]
pub enum Command {
    Empty,
    Unknown(String),
    InvalidArgs(String),
    Exit,
    Pwd,
    Cd(String),
    Echo(String),
    Type(String),
    External {
        program: String,
        args: Vec<String>,
        redirect_stdout: Option<(String, bool)>,
        redirect_stderr: Option<(String, bool)>,
    },
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

    fn redirect_info(&self) -> Option<(bool, bool)> {
        match self {
            Token::RedirectStdout => Some((true, false)),
            Token::RedirectAppendStdout => Some((true, true)),
            Token::RedirectStderr => Some((false, false)),
            Token::RedirectAppendStderr => Some((false, true)),
            _ => None,
        }
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
        return Command::Empty;
    }

    // First Word is the command name
    let (cmd_name, rest) = match &tokens[0] {
        Token::Word(name) => (name.as_str(), &tokens[1..]),
        _ => return Command::Empty,
    };

    match cmd_name {
        "exit" => return Command::Exit,
        "pwd" => return Command::Pwd,
        "echo" => {
            return Command::Echo(words_from_tokens(rest.iter().collect()).join(" "));
        }
        "cd" => {
            return match rest.first() {
                Some(Token::Word(p)) => Command::Cd(p.to_string()),
                _ => Command::InvalidArgs(input.to_string()),
            };
        }
        "type" => {
            return match rest.first() {
                Some(Token::Word(n)) => Command::Type(n.to_string()),
                _ => Command::InvalidArgs(input.to_string()),
            };
        }
        _ => match parse_external(rest) {
            Some(parts) => Command::External {
                program: cmd_name.to_string(),
                args: parts.args,
                redirect_stdout: parts.redirect_stdout,
                redirect_stderr: parts.redirect_stderr,
            },
            None => Command::InvalidArgs(input.to_string()),
        },
    }
}

struct ExternalParts {
    args: Vec<String>,
    redirect_stdout: Option<(String, bool)>,
    redirect_stderr: Option<(String, bool)>,
}

fn parse_external(tokens: &[Token]) -> Option<ExternalParts> {
    let mut args = Vec::new();
    let mut redirect = None;
    let mut file = None;

    for token in tokens {
        match token {
            Token::Word(w) if redirect.is_none() => args.push(w.clone()),
            t if t.is_redirect() && redirect.is_none() => redirect = Some(t),
            Token::Word(w) if redirect.is_some() && file.is_none() => file = Some(w.clone()),
            _ => return None,
        }
    }

    let (redirect_stdout, redirect_stderr) = match redirect.and_then(|r| r.redirect_info()) {
        Some((true, append)) => (file.map(|f| (f, append)), None),
        Some((false, append)) => (None, file.map(|f| (f, append))),
        None => (None, None),
    };

    Some(ExternalParts {
        args,
        redirect_stdout,
        redirect_stderr,
    })
}

fn words_from_tokens(tokens: Vec<&Token>) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Word(w) => Some(w.clone()),
            _ => None,
        })
        .collect()
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
