use super::*;

fn assert_command(input: &str, expected: Command) {
    assert_eq!(parse(input), expected);
}

// ══════════════════════════════════════════════════════════════
// Builtins
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_exit() {
    assert_command("exit", Command::Exit);
}

#[test]
fn parse_pwd() {
    assert_command("pwd", Command::Pwd);
}

// ── Echo ──

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
fn parse_echo_with_quotes() {
    assert_command("echo 'hello world'", Command::Echo("hello world".into()));
}

#[test]
fn parse_echo_with_escapes() {
    assert_command(r"echo hello\ world", Command::Echo("hello world".into()));
}

// ── Cd ──

#[test]
fn parse_cd_no_args() {
    assert_command("cd", Command::InvalidArgs("cd".into()));
}

#[test]
fn parse_cd_absolute() {
    assert_command("cd /tmp", Command::Cd("/tmp".into()));
}

#[test]
fn parse_cd_relative() {
    assert_command("cd ..", Command::Cd("..".into()));
}

#[test]
fn parse_cd_home() {
    assert_command("cd ~", Command::Cd("~".into()));
}

#[test]
fn parse_cd_extra_args_ignored() {
    assert_command("cd /tmp extra", Command::Cd("/tmp".into()));
}

// ── Type ──

#[test]
fn parse_type_no_args() {
    assert_command("type", Command::InvalidArgs("type".into()));
}

#[test]
fn parse_type_builtin() {
    assert_command("type echo", Command::Type("echo".into()));
}

#[test]
fn parse_type_external() {
    assert_command("type ls", Command::Type("ls".into()));
}

// ══════════════════════════════════════════════════════════════
// External commands
// ══════════════════════════════════════════════════════════════

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
fn parse_external_one_arg() {
    assert_command(
        "cat file.txt",
        Command::External {
            program: "cat".into(),
            args: vec!["file.txt".into()],
            redirect_stdout: None,
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_external_multiple_args() {
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
fn parse_external_with_quoted_arg() {
    assert_command(
        r#"grep "hello world" file.txt"#,
        Command::External {
            program: "grep".into(),
            args: vec!["hello world".into(), "file.txt".into()],
            redirect_stdout: None,
            redirect_stderr: None,
        },
    );
}

// ══════════════════════════════════════════════════════════════
// Redirects
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_redirect_stdout() {
    assert_command(
        "echo hello > out.txt",
        Command::External {
            program: "echo".into(),
            args: vec!["hello".into()],
            redirect_stdout: Some(("out.txt".into(), false)),
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_redirect_stdout_append() {
    assert_command(
        "echo hello >> out.txt",
        Command::External {
            program: "echo".into(),
            args: vec!["hello".into()],
            redirect_stdout: Some(("out.txt".into(), true)),
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_redirect_stderr() {
    assert_command(
        "ls 2> errors.txt",
        Command::External {
            program: "ls".into(),
            args: vec![],
            redirect_stdout: None,
            redirect_stderr: Some(("errors.txt".into(), false)),
        },
    );
}

#[test]
fn parse_redirect_stderr_append() {
    assert_command(
        "ls 2>> errors.txt",
        Command::External {
            program: "ls".into(),
            args: vec![],
            redirect_stdout: None,
            redirect_stderr: Some(("errors.txt".into(), true)),
        },
    );
}

#[test]
fn parse_redirect_stdout_fd1() {
    assert_command(
        "echo hello 1> out.txt",
        Command::External {
            program: "echo".into(),
            args: vec!["hello".into()],
            redirect_stdout: Some(("out.txt".into(), false)),
            redirect_stderr: None,
        },
    );
}

#[test]
fn parse_redirect_both() {
    assert_command(
        "cmd > out.txt 2> err.txt",
        Command::External {
            program: "cmd".into(),
            args: vec![],
            redirect_stdout: Some(("out.txt".into(), false)),
            redirect_stderr: Some(("err.txt".into(), false)),
        },
    );
}

// ══════════════════════════════════════════════════════════════
// Edge cases
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_empty_input() {
    assert_command("", Command::Empty);
}

#[test]
fn parse_whitespace_only() {
    assert_command("   ", Command::Empty);
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

#[test]
fn parse_leading_whitespace() {
    assert_command("  pwd", Command::Pwd);
}

#[test]
fn parse_trailing_whitespace() {
    assert_command("pwd  ", Command::Pwd);
}
