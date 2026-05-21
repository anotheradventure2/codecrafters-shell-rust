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
    External(String, Vec<String>),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),         // regular word or quoted string
    RedirectStdout,       // >
    RedirectStdoutAppend, // >>
    RedirectStderr,       // 2>
    RedirectStderrAppend, // 2>>
    Pipe,
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
            return Command::Echo(words_from_tokens(&tokens).join(" "));
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
            return Command::External(cmd_name.to_string(), words_from_tokens(&tokens));
        }
    }
}

fn words_from_tokens(tokens: &[Token]) -> Vec<String> {
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
}
