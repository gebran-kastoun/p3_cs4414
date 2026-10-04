// Integration tests: build the `minishell` binary, feed it commands on stdin,
// and check its observable behavior.
// Run a single milestone with, e.g., `cargo test s3`.

use assert_cmd::Command;
use std::path::PathBuf;

/// A unique temp file path for redirection tests (no spaces in the path).
fn temp_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("minishell_{tag}_{nanos}.txt"));
    path
}

fn shell() -> Command {
    Command::cargo_bin("minishell").expect("binary builds")
}

// Step 2

#[test]
fn s2_check_pwd() {
    let dir = std::env::current_dir().expect("Run from valid directory");
    shell()
        .write_stdin("pwd\n")
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("{}", dir.display())));
}

#[test]
fn s2_check_cd() {
    let mut dir = std::env::current_dir().expect("Run from valid directory");
    dir.push("src");
    shell()
        .write_stdin("cd src\npwd\n")
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("{}", dir.display())));
}

#[test]
fn s2_check_source() {
    let path = temp_path("out");
    let _ = std::fs::remove_file(&path);
    std::fs::write(&path, "cd src\npwd\n").expect("Write temp file");
    let mut dir = std::env::current_dir().expect("Run from valid directory");
    dir.push("src");
    shell()
        .write_stdin(format!("source {}\n", path.display()))
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("{}", dir.display())));
    let _ = std::fs::remove_file(&path);
}

// Step 3

#[test]
fn s3_runs_external_command() {
    shell()
        .write_stdin("echo hello\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn s3_survives_failing_command_and_keeps_going() {
    // A command that exits non-zero must not crash the shell; the next command
    // should still run.
    shell()
        .write_stdin("false\necho after\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("after"));
}

#[test]
fn s3_missing_command_does_not_crash_shell() {
    // A command that isn't found (as opposed to one that runs and exits
    // non-zero, like `false`) exercises the `Command::spawn()` error path
    // specifically. The shell must report the problem and keep going.
    shell()
        .write_stdin("not_a_real_command\necho after\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("after"));
}

// Step 4: pipelines

#[test]
fn s4_pipes_output_between_commands() {
    // "hello\n" is 6 bytes; piping it into `wc -c` prints 6.
    shell()
        .write_stdin("echo hello | wc -c\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("6"));
}

#[test]
fn s4_three_stage_pipeline() {
    // Three lines in, `grep b` keeps one, `wc -l` counts 1.
    shell()
        .write_stdin("printf 'a\\nb\\nc\\n' | grep b | wc -l\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("1"));
}

// Step 5

#[test]
fn s5_output_redirection_writes_file() {
    let path = temp_path("out");
    let _ = std::fs::remove_file(&path);

    shell()
        .write_stdin(format!("echo hello > {}\nexit\n", path.display()))
        .assert()
        .success();

    let contents = std::fs::read_to_string(&path).expect("file was created");
    assert_eq!(contents, "hello\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn s5_append_redirection_adds_to_file() {
    let path = temp_path("append");
    std::fs::write(&path, "first\n").unwrap();

    shell()
        .write_stdin(format!("echo second >> {}\nexit\n", path.display()))
        .assert()
        .success();

    let contents = std::fs::read_to_string(&path).expect("file exists");
    assert_eq!(contents, "first\nsecond\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn s5_input_redirection_reads_file() {
    let path = temp_path("in");
    std::fs::write(&path, "abcdef").unwrap();

    shell()
        .write_stdin(format!("wc -c < {}\nexit\n", path.display()))
        .assert()
        .success()
        .stdout(predicates::str::contains("6"));

    let _ = std::fs::remove_file(&path);
}

// Step 6

#[test]
fn s6_input_redirection_reads_url() {
    shell()
        .write_stdin("wc -l < https://www.cs.cornell.edu/courses/cs4414/2026fa/lab/01-intro.html\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("760"));
}

#[test]
fn s6_input_from_url_with_pipeline() {
    shell()
        .write_stdin("cat < https://www.cs.cornell.edu/courses/cs4414/2026fa/lab/01-intro.html | wc -l\nexit\n")
        .assert()
        .success()
        .stdout(predicates::str::contains("760"));
}
