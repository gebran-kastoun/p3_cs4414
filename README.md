# CS4414 HW 3 - Minishell
## Logistics

- This homework may be done individually or with a partner (recommended)
- 
- The full submission is due Mon, Oct 19 at 11:59 PM via Gradescope

## Changelog
2026-09-28:
- Initial project release

## Introduction
Operating systems are system softwares that handle several important services that are all handled by the core computer program, the kernel. Amongst these services are hardware and software resource management, file management, prcess management, etc. A computer program called the shell allows users to communicate with the system, translating user commands and launching other programs/processes accordingly.

For this assignment, we will build a small Unix command-line shell that will print a prompt, reads user input on the standard input stream, and executes processes required for the user command.

## Setup
All the directions for this assignment are in this file. The starter code is split across two files: `src/lib.rs` (where the parsing logic is) and `src/main.rs` (the interactive shell driver). 

Clone the repo and build with `cargo build` and make sure [rustup](https://rustup.rs/) is installed. Code submitted to Gradescope will be checked against the milestone test suite in `tests/`. Please do not modify anything in that directory as the autograder uses its own copy regardless of what you submit there.
Additionally, there are questions for you to fill out in M2; make sure to fill them out in the `WRITEUP.md` file.
For testing, run `cargo test [m#_]` and then `cargo run` to run the shell.

## Background

### Shell lifecycle: REPL
The shell is a looping computer program that follows read-eval-print approach (REPL): print a prompt, read user input, translates user input, executes required processes, and repeats. 

### Running programs: fork() and exec()
When running programs, two fundamental system calls are used: `fork()` and `exec()`. Here, `fork` clones the current process, creating a child duplicate of the current process, while `exec` replaces the current process with a new program. The parent, which is the shell, then waits for the child to finish.

`std::process::Command` wraps all of that:

```rust
use std::process::Command;

let mut child = Command::new("ls")     // the program to run
    .arg("-la")                        // add one argument (or .args([...]))
    .spawn()?;                         // fork + exec, returns a Child handle
let status = child.wait()?;            // block until it finishes
let code: i32 = status.code().unwrap_or(0);  // its exit code
```
### Why `cd` has to be built-in
When the operating system creates a process, this process gets its own copy of the current directory (own private working directory). If we were run a shell (where the current working directory is /home/you, and we type `cd /temp`. Following the previous section, the shell forks, making a child copy of itself, and executes, loading the cd program into the child process. This same child process will call the instruction that changes the directory to `/temp`, change its own current directory to `/temp`, and exit; this will change nothing about the parent, our shell. 

The only process that can change the shell's current working directory is the shell itself with no calls to fork, forcing it to be built-in. exit is built-in for a similar reason: it has to stop the shell's own loop, and a child process cannot reach up and do that.  

#### Standard streams, pipes, and `Stdio`

Every process gets three streams: 
- standard input (`stdin`),
- standard output (`stdout`), and
- standard error (`stderr`).


A pipe joins one process' `stdout` to the next process' `stdin`. This pipe is represented with a `|`:

```text
echo hello  --stdout-->  [pipe]  --stdin-->  wc -c
```

In Rust you set up a child's streams before you spawn it, using `Stdio`:

```rust
use std::process::Stdio;

Stdio::inherit()      // reuse the shell's own stream (goes to the terminal)
Stdio::piped()        // make a pipe; read it later through child.stdout
Stdio::from(x)        // use an already-open stream or file `x`
```

Chaining commands means grabbing the previous child's captured output and
feeding it into the next one:

```rust
let mut child = Command::new(prog)
    .stdin(previous_stdout)   // Stdio::from(prev) or Stdio::inherit()
    .stdout(Stdio::piped())   // capture it so the next stage can read it
    .spawn()?;
let prev_stdout = child.stdout.take();  // Option<ChildStdout> for the next stage
```

#### Redirection to and from files

Redirection swaps a stream for a file. Output lands in a file instead of the terminal while input comes from a file instead of the keyboard:

```rust
use std::fs::{File, OpenOptions};

let out = File::create("out.txt")?;                    // `>`  truncate/create
let app = OpenOptions::new().create(true)
    .append(true).open("log.txt")?;                    // `>>` append/create
let inp = File::open("in.txt")?;                       // `<`  read

// then hand any of these to the command:
Command::new("sort").stdin(Stdio::from(inp)).stdout(Stdio::from(out)).spawn()?;
```
## Milestones

### Milestone 1
Fill in two functions in [`src/lib.rs`](src/lib.rs).
#### `tokenize`

```rust
pub fn tokenize(input: &str) -> Vec<String>
```

Split the line into whitespace-separated tokens. In this assignment, the operators
`<`, `>`, `>>`, and `|` always have spaces around them, so splitting on
whitespace is all what's need.

- `tokenize("ls -la")` -> `["ls", "-la"]`
- `tokenize("echo hi > out")` -> `["echo", "hi", ">", "out"]`
- `tokenize("   ")` -> `[]`

#### `parse_command` and `parse_pipeline`

```rust
pub fn parse_command(tokens: &[String]) -> Result<Command, ParseError>
pub fn parse_pipeline(input: &str) -> Result<Pipeline, ParseError>
```

`parse_command` puts the first token in `tokens` in 
`program` and the rest in `args` for `Command`. This function returns `ParseError::EmptyCommand` when the line is blank. 
`parse_pipeline` returns a `Pipeline` holding a
single stage; otherwise, it returns`ParseError::EmptyStage` when a stage has no program. You will come back to both functions in M4 and M5.

### Milestone 2
Implement the built-in `cd` in `src/main.rs`. The other two, `exit` and `pwd` are already provided; read these two closely as `cd` follows the same shape.

#### cd
​```rust
pub fn cd(args: &[String]) -> Result<(), std::io::Error>
​```
With 0 arguments, `cd` should go to the user's home directory by reading the
`HOME` environment variable. With one argument, change to that directory using `std::env::set_current_dir`. If the directory doesn't exist or can't be
entered, print an error message and keep the shell running. Don't panic and
don't change the current working directory.

- `cd` (no args) -> moves to `$HOME`
- `cd /temp` -> moves to `/temp`
- `cd /nonexistent` -> prints an error, shell keeps running, current working directory is unchanged

#### `exit` and `pwd`
`exit`, `pwd`, `as_builtin` in `src/lib.rs` and the built-in dispatch in `src/main.rs` are implemented for you. Read the code and make sure you can answer:
- how `as_builtin` picks out `cd` / `exit` / `pwd`,
- why `exit` returns out of `main` instead of spawning anything
- *insert another question*

Answer these three questions in `WRITEUP.md`.

### Milestone 3
Implement `run_pipeline` in [`src/main.rs`](src/main.rs) for the single-stage case: build and spawn a `Command`, wait for it to run, and return its exit code. The file already has a skeleton mapped out for you with `prev_stdout` / `children` bookkeeping and step-by-step comments.
Note, a missing or failing command should not crash the shell or take it down. For instance, if command is not found or exits non-zero, print a message or exit code and prompt again.

### Milestone 4
Now, we will extend both the parser and the executor. 
#### Parsing
```rust
pub fn parse_pipeline(_input: &str) -> Result<Pipeline, ParseError> {
```
`parse_pipeline` splits `_input` on `|` and parses each individual part. For instance, `"ls -la | wc -l"` becomes two `Command`s. Treat incomplete inputs like `ls |`, `| ls`, and `a | | b` as `ParseError::EmptyStage`.

#### Execution
- Now we will connect the stages. Every stage except the last one sends its stdout into a `Stdio::piped()`, and the next stage reads that through
`Stdio::from(prev_stdout)`. The last stage then inherits the shell's stdout. Make sure to wait for all of the children, not just the last.

>  Note: the classic bug here is a pipeline that just hangs. Give a stage a
> `piped()` stdout only when there is a next stage to read it, and make sure you
> `take()` the child's stdout so it is handed to exactly one reader.

### Milestone 5
Now we want to teach the parser to recognize redirection operators inside a state and teach the executor to obey them.

- `< file` sets `Command.stdin` to that file name.
- `> file` sets `Command.stdout` to `(file, RedirectMode::Truncate)`.
- `>> file` sets `Command.stdout` to `(file, RedirectMode::Append)`.
- an operator with no file after it is `ParseError::MissingRedirectTarget`.

In `run_pipeline`, a stage's own redirection wins over the pipe. 
- If `stage.stdin` is set, open that file for input.
- If `stage.stdout` is set, open or create that file for output, truncating or appending to match the mode.

## Rules

1. Edit `src/lib.rs` and `src/main.rs` only. Do not touch anything in `tests/`.
2. Keep parsing in the library and execution in the binary. Do not spawn
   processes from `src/lib.rs`.
3. You may use the Rust standard library (`std::process`, `std::fs`, `std::env`,
   `std::io`). Do not hand the work off to a real shell (`sh -c`, `system`, or a
   `popen`-style wrapper).
4. Built-ins (`cd`, `exit`, `pwd`) run in the shell process, never as children.
5. A failing or unknown command must not crash the shell.
