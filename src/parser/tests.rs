use super::*;

fn cmd(kind: CommandKind) -> Command {
    Command {
        kind,
        redirect_stdout: None,
        redirect_stderr: None,
    }
}

fn cmd_redirect(
    kind: CommandKind,
    stdout: Option<(&str, bool)>,
    stderr: Option<(&str, bool)>,
) -> Command {
    Command {
        kind,
        redirect_stdout: stdout.map(|(f, a)| (f.into(), a)),
        redirect_stderr: stderr.map(|(f, a)| (f.into(), a)),
    }
}

fn assert_command(input: &str, expected: Command) {
    assert_eq!(parse(input), expected);
}

// ══════════════════════════════════════════════════════════════
// Builtins
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_exit() {
    assert_command("exit", cmd(CommandKind::Exit));
}

#[test]
fn parse_pwd() {
    assert_command("pwd", cmd(CommandKind::Pwd));
}

// ── Echo ──

#[test]
fn parse_echo_no_args() {
    assert_command("echo", cmd(CommandKind::Echo(String::new())));
}

#[test]
fn parse_echo_one_arg() {
    assert_command("echo hello", cmd(CommandKind::Echo("hello".into())));
}

#[test]
fn parse_echo_multiple_args() {
    assert_command(
        "echo hello world",
        cmd(CommandKind::Echo("hello world".into())),
    );
}

#[test]
fn parse_echo_with_quotes() {
    assert_command(
        "echo 'hello world'",
        cmd(CommandKind::Echo("hello world".into())),
    );
}

#[test]
fn parse_echo_with_escapes() {
    assert_command(
        r"echo hello\ world",
        cmd(CommandKind::Echo("hello world".into())),
    );
}

// ── Cd ──

#[test]
fn parse_cd_no_args() {
    assert_command("cd", cmd(CommandKind::InvalidArgs("cd".into())));
}

#[test]
fn parse_cd_absolute() {
    assert_command("cd /tmp", cmd(CommandKind::Cd("/tmp".into())));
}

#[test]
fn parse_cd_relative() {
    assert_command("cd ..", cmd(CommandKind::Cd("..".into())));
}

#[test]
fn parse_cd_home() {
    assert_command("cd ~", cmd(CommandKind::Cd("~".into())));
}

#[test]
fn parse_cd_extra_args_ignored() {
    assert_command("cd /tmp extra", cmd(CommandKind::Cd("/tmp".into())));
}

// ── Type ──

#[test]
fn parse_type_no_args() {
    assert_command("type", cmd(CommandKind::InvalidArgs("type".into())));
}

#[test]
fn parse_type_builtin() {
    assert_command("type echo", cmd(CommandKind::Type("echo".into())));
}

#[test]
fn parse_type_external() {
    assert_command("type ls", cmd(CommandKind::Type("ls".into())));
}

// ══════════════════════════════════════════════════════════════
// External commands
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_external_no_args() {
    assert_command(
        "ls",
        cmd(CommandKind::External {
            program: "ls".into(),
            args: vec![],
        }),
    );
}

#[test]
fn parse_external_one_arg() {
    assert_command(
        "cat file.txt",
        cmd(CommandKind::External {
            program: "cat".into(),
            args: vec!["file.txt".into()],
        }),
    );
}

#[test]
fn parse_external_multiple_args() {
    assert_command(
        "ls -la /tmp",
        cmd(CommandKind::External {
            program: "ls".into(),
            args: vec!["-la".into(), "/tmp".into()],
        }),
    );
}

#[test]
fn parse_external_with_quoted_arg() {
    assert_command(
        r#"grep "hello world" file.txt"#,
        cmd(CommandKind::External {
            program: "grep".into(),
            args: vec!["hello world".into(), "file.txt".into()],
        }),
    );
}

// ══════════════════════════════════════════════════════════════
// Redirects
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_redirect_stdout() {
    assert_command(
        "ls > out.txt",
        cmd_redirect(
            CommandKind::External {
                program: "ls".into(),
                args: vec![],
            },
            Some(("out.txt", false)),
            None,
        ),
    );
}

#[test]
fn parse_redirect_stdout_append() {
    assert_command(
        "ls >> out.txt",
        cmd_redirect(
            CommandKind::External {
                program: "ls".into(),
                args: vec![],
            },
            Some(("out.txt", true)),
            None,
        ),
    );
}

#[test]
fn parse_redirect_stderr() {
    assert_command(
        "ls 2> errors.txt",
        cmd_redirect(
            CommandKind::External {
                program: "ls".into(),
                args: vec![],
            },
            None,
            Some(("errors.txt", false)),
        ),
    );
}

#[test]
fn parse_redirect_stderr_append() {
    assert_command(
        "ls 2>> errors.txt",
        cmd_redirect(
            CommandKind::External {
                program: "ls".into(),
                args: vec![],
            },
            None,
            Some(("errors.txt", true)),
        ),
    );
}

#[test]
fn parse_redirect_stdout_fd1() {
    assert_command(
        "ls 1> out.txt",
        cmd_redirect(
            CommandKind::External {
                program: "ls".into(),
                args: vec![],
            },
            Some(("out.txt", false)),
            None,
        ),
    );
}

#[test]
fn parse_redirect_with_args() {
    assert_command(
        "grep foo > out.txt",
        cmd_redirect(
            CommandKind::External {
                program: "grep".into(),
                args: vec!["foo".into()],
            },
            Some(("out.txt", false)),
            None,
        ),
    );
}

#[test]
fn parse_redirect_both() {
    assert_command(
        "cmd > out.txt 2> err.txt",
        cmd_redirect(
            CommandKind::External {
                program: "cmd".into(),
                args: vec![],
            },
            Some(("out.txt", false)),
            Some(("err.txt", false)),
        ),
    );
}

// ══════════════════════════════════════════════════════════════
// Edge cases
// ══════════════════════════════════════════════════════════════

#[test]
fn parse_empty_input() {
    assert_command("", cmd(CommandKind::Empty));
}

#[test]
fn parse_whitespace_only() {
    assert_command("   ", cmd(CommandKind::Empty));
}

#[test]
fn parse_unknown_becomes_external() {
    assert_command(
        "nonexistent",
        cmd(CommandKind::External {
            program: "nonexistent".into(),
            args: vec![],
        }),
    );
}

#[test]
fn parse_leading_whitespace() {
    assert_command("  pwd", cmd(CommandKind::Pwd));
}

#[test]
fn parse_trailing_whitespace() {
    assert_command("pwd  ", cmd(CommandKind::Pwd));
}
