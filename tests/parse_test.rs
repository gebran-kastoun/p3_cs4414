// Unit tests for the parsing library (`src/lib.rs`).
// Run a single milestone with, e.g., `cargo test s1`.

use minishell::{as_builtin, parse_pipeline, tokenize, Builtin, Command, ParseError, RedirectMode};

// Step 1

#[test]
fn s1_tokenize_splits_on_whitespace() {
    assert_eq!(tokenize("ls -la"), vec!["ls", "-la"]);
    assert_eq!(tokenize("  echo   hi  "), vec!["echo", "hi"]);
    assert!(tokenize("   ").is_empty());
}

#[test]
fn s1_parse_single_command() {
    let pipeline = parse_pipeline("ls -la").expect("valid command");
    assert_eq!(pipeline.stages.len(), 1);
    assert_eq!(
        pipeline.stages[0],
        Command {
            program: "ls".to_string(),
            args: vec!["-la".to_string()],
            stdin: None,
            stdout: None,
        }
    );
}

#[test]
fn s1_parse_empty_line_is_error() {
    assert_eq!(parse_pipeline("   "), Err(ParseError::EmptyCommand));
}

// Step 2

#[test]
fn s2_as_builtin_classifies_names() {
    assert_eq!(as_builtin("cd"), Some(Builtin::Cd));
    assert_eq!(as_builtin("exit"), Some(Builtin::Exit));
    assert_eq!(as_builtin("pwd"), Some(Builtin::Pwd));
    assert_eq!(as_builtin("ls"), None);
    assert_eq!(as_builtin("echo"), None);
}

// Step 4

#[test]
fn s4_parse_two_stage_pipeline() {
    let pipeline = parse_pipeline("ls -la | wc -l").expect("valid pipeline");
    assert_eq!(pipeline.stages.len(), 2);
    assert_eq!(pipeline.stages[0].program, "ls");
    assert_eq!(pipeline.stages[0].args, vec!["-la"]);
    assert_eq!(pipeline.stages[1].program, "wc");
    assert_eq!(pipeline.stages[1].args, vec!["-l"]);
}

#[test]
fn s4_parse_empty_stage_is_error() {
    assert_eq!(parse_pipeline("ls | | wc"), Err(ParseError::EmptyStage));
    assert_eq!(parse_pipeline("ls |"), Err(ParseError::EmptyStage));
}

// Step 5

#[test]
fn s5_parse_output_redirect_truncate() {
    let pipeline = parse_pipeline("echo hi > out.txt").expect("valid");
    let stage = &pipeline.stages[0];
    assert_eq!(stage.program, "echo");
    assert_eq!(stage.args, vec!["hi"]);
    assert_eq!(
        stage.stdout,
        Some(("out.txt".to_string(), RedirectMode::Truncate))
    );
    assert_eq!(stage.stdin, None);
}

#[test]
fn s5_parse_output_redirect_append() {
    let pipeline = parse_pipeline("echo hi >> log").expect("valid");
    assert_eq!(
        pipeline.stages[0].stdout,
        Some(("log".to_string(), RedirectMode::Append))
    );
}

#[test]
fn s5_parse_input_redirect() {
    let pipeline = parse_pipeline("wc -l < in.txt").expect("valid");
    let stage = &pipeline.stages[0];
    assert_eq!(stage.program, "wc");
    assert_eq!(stage.args, vec!["-l"]);
    assert_eq!(stage.stdin, Some("in.txt".to_string()));
    assert_eq!(stage.stdout, None);
}

#[test]
fn s5_parse_missing_redirect_target_is_error() {
    assert_eq!(
        parse_pipeline("echo hi >"),
        Err(ParseError::MissingRedirectTarget)
    );
}
