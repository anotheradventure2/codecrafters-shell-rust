use super::*;

// ── Helpers ──

macro_rules! w {
    ($s:literal) => {
        Token::Word($s.into())
    };
}

fn assert_tokens(input: &str, expected: Vec<Token>) {
    assert_eq!(tokenize(input), expected);
}

fn assert_command(input: &str, expected: Command) {
    assert_eq!(parse(input), expected);
}

// ══════════════════════════════════════════════════════════════
// Tokenizer tests
// ══════════════════════════════════════════════════════════════

// ── Basic word splitting ──
#[test]
fn tokenize_simple() {
    assert_tokens("echo hello", vec![w!("echo"), w!("hello")]);
}
#[test]
fn tokenize_three_words() {
    assert_tokens("ls -la /tmp", vec![w!("ls"), w!("-la"), w!("/tmp")]);
}
#[test]
fn tokenize_single_word() {
    assert_tokens("pwd", vec![w!("pwd")]);
}

// ── Empty / whitespace ──
#[test]
fn tokenize_empty() {
    assert_tokens("", vec![]);
}
#[test]
fn tokenize_only_spaces() {
    assert_tokens("   ", vec![]);
}
#[test]
fn tokenize_leading_space() {
    assert_tokens("  echo", vec![w!("echo")]);
}
#[test]
fn tokenize_trailing_space() {
    assert_tokens("echo ", vec![w!("echo")]);
}
#[test]
fn tokenize_multiple_spaces() {
    assert_tokens("echo   hello", vec![w!("echo"), w!("hello")]);
}

// ── Single quotes ──
#[test]
fn tokenize_single_quote() {
    assert_tokens("echo 'hello world'", vec![w!("echo"), w!("hello world")]);
}
#[test]
fn tokenize_empty_quotes() {
    assert_tokens("echo ''", vec![w!("echo")]);
}
#[test]
fn tokenize_quote_with_special() {
    assert_tokens("echo 'a>b|c'", vec![w!("echo"), w!("a>b|c")]);
}
#[test]
fn tokenize_backslash_in_single() {
    assert_tokens(r"echo 'hello\world'", vec![w!("echo"), w!(r"hello\world")]);
}

// ── Double quotes ──
#[test]
fn tokenize_double_quote() {
    assert_tokens(r#"echo "hello world""#, vec![w!("echo"), w!("hello world")]);
}
#[test]
fn tokenize_double_empty() {
    assert_tokens(r#"echo """#, vec![w!("echo")]);
}
#[test]
fn tokenize_escaped_quote() {
    assert_tokens(
        r#"echo "hello\"world""#,
        vec![w!("echo"), w!(r#"hello"world"#)],
    );
}
#[test]
fn tokenize_escaped_backslash() {
    assert_tokens(
        r#"echo "hello\\world""#,
        vec![w!("echo"), w!(r#"hello\world"#)],
    );
}

// ── Backslash outside quotes ──
#[test]
fn tokenize_escape_space() {
    assert_tokens(r"hello\ world", vec![w!("hello world")]);
}
#[test]
fn tokenize_escape_quote() {
    assert_tokens(r"hello\'world", vec![w!("hello'world")]);
}
#[test]
fn tokenize_escape_dquote() {
    assert_tokens(r#"hello\"world"#, vec![w!(r#"hello"world"#)]);
}
#[test]
fn tokenize_escape_backslash() {
    assert_tokens(r"hello\\world", vec![w!(r"hello\world")]);
}
#[test]
fn tokenize_trailing_backslash() {
    assert_tokens(r"hello\", vec![w!("hello")]);
}

// ── Mixed quotes ──
#[test]
fn tokenize_single_in_double() {
    assert_tokens(r#"echo "it's fine""#, vec![w!("echo"), w!("it's fine")]);
}

// ── Edge cases ──
#[test]
fn tokenize_unclosed_single() {
    assert_tokens("echo 'hello", vec![w!("echo"), w!("hello")]);
}
#[test]
fn tokenize_unclosed_double() {
    assert_tokens(r#"echo "hello"#, vec![w!("echo"), w!("hello")]);
}
#[test]
fn tokenize_only_quotes() {
    assert_tokens(r"''", vec![]);
}
#[test]
fn tokenize_adjacent_quotes() {
    assert_tokens(r"echo a''b", vec![w!("echo"), w!("ab")]);
}

// ── Redirects ──
#[test]
fn tokenize_redirect_stdout() {
    assert_tokens(
        "echo > file",
        vec![w!("echo"), Token::RedirectStdout, w!("file")],
    );
}
#[test]
fn tokenize_redirect_append() {
    assert_tokens(
        "echo >> file",
        vec![w!("echo"), Token::RedirectAppendStdout, w!("file")],
    );
}
#[test]
fn tokenize_redirect_stderr() {
    assert_tokens(
        "echo 2> file",
        vec![w!("echo"), Token::RedirectStderr, w!("file")],
    );
}
#[test]
fn tokenize_redirect_stdout_fd1() {
    assert_tokens(
        "echo 1> file",
        vec![w!("echo"), Token::RedirectStdout, w!("file")],
    );
}

// ══════════════════════════════════════════════════════════════
// Parser tests
// ══════════════════════════════════════════════════════════════

// ── Builtins ──
#[test]
fn parse_exit() {
    assert_command("exit", Command::Exit);
}

#[test]
fn parse_pwd() {
    assert_command("pwd", Command::Pwd);
}

#[test]
fn parse_echo_no_args() {
    assert_command("echo", Command::Echo(String::new()));
}

#[test]
fn parse_echo_one_arg() {
    assert_command("echo hello", Command::Echo("hello".into()));
}

#[test]
fn parse_echo_multiple_args() {
    assert_command("echo hello world", Command::Echo("hello world".into()));
}

#[test]
fn parse_cd_no_args() {
    assert_command("cd", Command::InvalidArgs("cd".into()));
}

#[test]
fn parse_cd_with_path() {
    assert_command("cd /tmp", Command::Cd("/tmp".into()));
}

#[test]
fn parse_cd_extra_args_ignored() {
    assert_command("cd too many args", Command::Cd("too".into()));
}

#[test]
fn parse_type_no_args() {
    assert_command("type", Command::InvalidArgs("type".into()));
}

#[test]
fn parse_type_with_name() {
    assert_command("type echo", Command::Type("echo".into()));
}

// ── External commands ──
#[test]
fn parse_external_no_args() {
    assert_command(
        "ls",
        Command::External {
            program: "ls".into(),
            args: vec![],
            redirect_stdout: None,
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_external_with_args() {
    assert_command(
        "ls -la /tmp",
        Command::External {
            program: "ls".into(),
            args: vec!["-la".into(), "/tmp".into()],
            redirect_stdout: None,
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_unknown_becomes_external() {
    assert_command(
        "nonexistent",
        Command::External {
            program: "nonexistent".into(),
            args: vec![],
            redirect_stdout: None,
            redirect_stderr: None,
        },
    );
}

// ── Empty input ──
#[test]
fn parse_empty_input() {
    assert_command("", Command::Empty);
}
