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

pub fn tokenize(input: &str) -> Vec<Token> {
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
        _ => {
            let (args, redirect, file) = match parse_external(rest) {
                Some(result) => result,
                None => return Command::InvalidArgs(input.to_string()),
            };

            let filename = file.and_then(|f| match f {
                Token::Word(name) => Some(name.clone()),
                _ => None,
            });

            let (redirect_stdout, redirect_stderr) = match redirect.and_then(|r| r.redirect_info())
            {
                Some((true, append)) => (filename.map(|f| (f, append)), None),
                Some((false, append)) => (None, filename.map(|f| (f, append))),
                None => (None, None),
            };

            return Command::External {
                program: cmd_name.to_string(),
                args: words_from_tokens(args),
                redirect_stdout,
                redirect_stderr,
            };
        }
    }
}

fn parse_external(tokens: &[Token]) -> Option<(Vec<&Token>, Option<&Token>, Option<&Token>)> {
    let mut args = Vec::new();
    let mut redirect = None;
    let mut file = None;

    for token in tokens {
        match token {
            Token::Word(_) if redirect.is_none() => args.push(token),
            t if t.is_redirect() && redirect.is_none() => redirect = Some(t),
            Token::Word(_) if redirect.is_some() && file.is_none() => file = Some(token),
            _ => return Option::None,
        }
    }

    Some((args, redirect, file))
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
mod tests {
    use super::*;
    macro_rules! w {
        ($s:literal) => {
            Token::Word($s.into())
        };
    }
    fn assert_tokens(input: &str, expected: Vec<Token>) {
        assert_eq!(tokenize(input), expected);
    }
    // ── Basic word splitting ──
    #[test]
    fn test_simple() {
        assert_tokens("echo hello", vec![w!("echo"), w!("hello")]);
    }
    #[test]
    fn test_three_words() {
        assert_tokens("ls -la /tmp", vec![w!("ls"), w!("-la"), w!("/tmp")]);
    }
    #[test]
    fn test_single_word() {
        assert_tokens("pwd", vec![w!("pwd")]);
    }
    // ── Empty / whitespace ──
    #[test]
    fn test_empty() {
        assert_tokens("", vec![]);
    }
    #[test]
    fn test_only_spaces() {
        assert_tokens("   ", vec![]);
    }
    #[test]
    fn test_leading_space() {
        assert_tokens("  echo", vec![w!("echo")]);
    }
    #[test]
    fn test_trailing_space() {
        assert_tokens("echo ", vec![w!("echo")]);
    }
    #[test]
    fn test_multiple_spaces() {
        assert_tokens("echo   hello", vec![w!("echo"), w!("hello")]);
    }
    // ── Single quotes ──
    #[test]
    fn test_single_quote() {
        assert_tokens("echo 'hello world'", vec![w!("echo"), w!("hello world")]);
    }
    #[test]
    fn test_empty_quotes() {
        assert_tokens("echo ''", vec![w!("echo")]);
    }
    #[test]
    fn test_quote_with_special() {
        assert_tokens("echo 'a>b|c'", vec![w!("echo"), w!("a>b|c")]);
    }
    #[test]
    fn test_backslash_in_single() {
        assert_tokens(r"echo 'hello\world'", vec![w!("echo"), w!(r"hello\world")]);
    }
    // ── Double quotes ──
    #[test]
    fn test_double_quote() {
        assert_tokens(r#"echo "hello world""#, vec![w!("echo"), w!("hello world")]);
    }
    #[test]
    fn test_double_empty() {
        assert_tokens(r#"echo """#, vec![w!("echo")]);
    }
    #[test]
    fn test_escaped_quote() {
        assert_tokens(
            r#"echo "hello\"world""#,
            vec![w!("echo"), w!(r#"hello"world"#)],
        );
    }
    #[test]
    fn test_escaped_backslash() {
        assert_tokens(
            r#"echo "hello\\world""#,
            vec![w!("echo"), w!(r#"hello\world"#)],
        );
    }
    // ── Backslash outside quotes ──
    #[test]
    fn test_escape_space() {
        assert_tokens(r"hello\ world", vec![w!("hello world")]);
    }
    #[test]
    fn test_escape_quote() {
        assert_tokens(r"hello\'world", vec![w!("hello'world")]);
    }
    #[test]
    fn test_escape_dquote() {
        assert_tokens(r#"hello\"world"#, vec![w!(r#"hello"world"#)]);
    }
    #[test]
    fn test_escape_backslash() {
        assert_tokens(r"hello\\world", vec![w!(r"hello\world")]);
    }
    #[test]
    fn test_trailing_backslash() {
        assert_tokens(r"hello\", vec![w!("hello")]);
    }
    // ── Mixed quotes ──
    #[test]
    fn test_single_in_double() {
        assert_tokens(r#"echo "it's fine""#, vec![w!("echo"), w!("it's fine")]);
    }
    // ── Edge cases ──
    #[test]
    fn test_unclosed_single() {
        assert_tokens("echo 'hello", vec![w!("echo"), w!("hello")]);
    }
    #[test]
    fn test_unclosed_double() {
        assert_tokens(r#"echo "hello"#, vec![w!("echo"), w!("hello")]);
    }
    #[test]
    fn test_only_quotes() {
        assert_tokens(r"''", vec![]);
    }
    #[test]
    fn test_adjacent_quotes() {
        assert_tokens(r"echo a''b", vec![w!("echo"), w!("ab")]);
    }

    #[test]
    fn test_redirect_stdout() {
        assert_tokens(
            "echo > file",
            vec![w!("echo"), Token::RedirectStdout, w!("file")],
        );
    }
    #[test]
    fn test_redirect_append() {
        assert_tokens(
            "echo >> file",
            vec![w!("echo"), Token::RedirectAppendStdout, w!("file")],
        );
    }
    #[test]
    fn test_redirect_stderr() {
        assert_tokens(
            "echo 2> file",
            vec![w!("echo"), Token::RedirectStderr, w!("file")],
        );
    }
    #[test]
    fn test_redirect_stdout_fd1() {
        assert_tokens(
            "echo 1> file",
            vec![w!("echo"), Token::RedirectStdout, w!("file")],
        );
    }
}
