# CS4414 HW 3 - Minishell

## Logistics

This homework may be done individually or with a partner
(recommended).  Step 1 is due by Mon, Oct 5 at 11:59 PM (Step 2 is
strongly recommended).  The full submission is due Mon, Oct 19 at
11:59 PM via Gradescope.

## Changelog

2026-09-29:
- Initial project release

## Introduction

For this assignment, we will build a small Unix command-line shell
that will print a prompt, reads user input on the standard input
stream, and run programs.

The directions for this assignment are in this file. The starter code
is split across two files: `src/lib.rs` (where the parsing logic is)
and `src/main.rs` (the interactive shell driver).  Code crates should
be submitted to Gradescope, where they will be checked against tests
in `tests/`. You may want to add your own tests, and that is fine.
The autograder uses its own copy regardless of what you submit there.
You may add functionality beyond what is specified, but you should not
break any of the interfaces in this assignment text.

For testing, run `cargo test [s#]` and then `cargo run` to run the
shell.

## Background

### Shell lifecycle: REPL

The shell is an interpreter that provides a read-eval-print loop
(REPL) interface: print a prompt, read user input, translates user
input, executes required processes, and repeats.

### Running programs: fork() and exec()

When running programs, two fundamental system calls are used: `fork()`
and `exec()`. Here, `fork` clones the current process, creating a
child duplicate of the current process, while `exec` replaces the
current process with a new program. The parent, which is the shell,
then waits for the child to finish.

`std::process::Command` wraps all of that:

```rust
use std::process::Command;

let mut child = Command::new("ls")     // the program to run
    .arg("-la")                        // add one argument (or .args([...]))
    .spawn()?;                         // fork + exec, returns a Child handle
let status = child.wait()?;            // block until it finishes
let code: i32 = status.code().unwrap_or(0);  // exit code (unless on signal)
```

### Why `cd` has to be built-in

Each new process has a current working directory.  That variable is
local to the process: if a shell process starts with the working
directory `/home/you` and forks a subprocess that changes its working
directory to `/tmp`, only that subprocess will be affected.  The
parent shell process will still have `/home/you` as its working
directory.

The only process that can change the shell's current working directory
is the shell itself with no calls to fork, forcing it to be
built-in. exit is built-in for a similar reason: it has to stop the
shell's own loop, and a child process cannot reach up and do that.

#### Standard file descriptors, pipes, and `Stdio`

Every process gets three file descriptors:

- standard input (`stdin`),
- standard output (`stdout`), and
- standard error (`stderr`).

We often think of `stdin` as associated with keyboard input and
`stdout` (and `stderr`) as associated with terminal output, but this
will not always be the case.

A pipe joins one process' `stdout` to the next process' `stdin`. This
pipe is represented with a `|`:

```text
echo hello  --stdout-->  [pipe]  --stdin-->  wc -c
```

Behind the scenes, child file descriptors are generally set up between when
a `fork` call spawns a new child process and when the child process
replaces itself with a new program using `exec`.  In Rust, the
`std::process::Stdio` functions help us with this setup:

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

In addition to redirecting the standard file descriptors to a pipe, we
can also redirect them to an ordinary file, either for input or for
output.

```rust
use std::fs::{File, OpenOptions};

let out = File::create("out.txt")?;                    // `>`  truncate/create
let app = OpenOptions::new().create(true)
    .append(true).open("log.txt")?;                    // `>>` append/create
let inp = File::open("in.txt")?;                       // `<`  read

// then hand any of these to the command:
Command::new("sort").stdin(Stdio::from(inp)).stdout(Stdio::from(out)).spawn()?;
```

#### Shell-supplied input (5416)

While command pipelines and redirection are the most common ways to
connect shell commands, the underlying mechanism is very flexible.
For example, many shells provide the notion of a "here doc" in which a
block of text in a shell script is fed to the standard input of a
program.  We will use that same mechanism to allow redirection from
remote resources specified by a URL, retrieved using the `reqwest`
crate.  Most shells do not bother with built-in support for redirection
from network resources, as the same effect can be achieved using
pipelines involving programs like `curl` or `wget`.

## Steps

### Step 1

Fill in two functions in [`src/lib.rs`](src/lib.rs).

#### `tokenize`

```rust
pub fn tokenize(input: &str) -> Vec<String>
```

Split the line into whitespace-separated tokens. In this assignment,
the operators `<`, `>`, `>>`, and `|` always have spaces around them,
so splitting on whitespace is all that's needed.

- `tokenize("ls -la")` -> `["ls", "-la"]`
- `tokenize("echo hi > out")` -> `["echo", "hi", ">", "out"]`
- `tokenize("   ")` -> `[]`

#### `parse_command` and `parse_pipeline`

```rust
pub fn parse_command(tokens: &[String]) -> Result<Command, ParseError>
pub fn parse_pipeline(input: &str) -> Result<Pipeline, ParseError>
```

`parse_command` puts the first token in `tokens` in `program` and the
rest in `args` for `Command`. This function returns
`ParseError::EmptyCommand` when the line is blank.  `parse_pipeline`
returns a `Pipeline` holding a single stage; otherwise, it
returns`ParseError::EmptyStage` when a stage has no program. You will
come back to both functions in steps 4 and 5.

### Step 2

Implement the built-in `cd` and `pwd` commands in `src/main.rs`. The
other two, `exit` and `pwd` are already provided; read these two
closely as `cd` follows the same shape.

#### cd

Fill in the `Some(Builtin::Cd)` arm in the built-in dispatch section
in `src/main.rs`. It should behave similarily in structure to the
`Pwd` arm below it.  With no arguments, `cd` should go to the user's
home directory by reading the `HOME` environment variable (see [S12.5
in the Rust book][rb-env]. With one argument, change to that directory
using `std::env::set_current_dir`. If the directory doesn't exist or
can't be entered, print an error message and keep the shell
running. Don't panic and don't change the current working directory.

- `cd` (no args) -> moves to `$HOME`
- `cd /tmp` -> moves to `/tmp`
- `cd /nonexistent` -> prints an error, shell keeps running, current
  working directory is unchanged

[rb-env]: https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html

#### `source`

Fill in the `Some(Builtin::Source)` arm in the built-in dispatch
section in `src/main.rs`.  This command should take the name of
exactly one file as an input.  If the file doesn't exist or can't be
read, print an error message and keep the shell running.  If the file
does exist and can be read, then call the `repl` function with
`is_interactive` set to false and a buffered reader around the file as
a command source.  This will read and execute the commands in the file
in the context of the current shell.  In the context of a sourced
file, you should interpret (and the default infrastructure does
interpret) `exit` to mean "stop executing commands from this file."

### Step 3

Implement `run_pipeline` in [`src/main.rs`](src/main.rs) for the
single-stage case: build and spawn a `Command`, wait for it to run,
and return its exit code. The file already has a skeleton mapped out
for you with `prev_stdout` / `children` bookkeeping and step-by-step
comments.

Note, a missing or failing command should not crash the shell or take
it down. For instance, if command is not found or exits non-zero,
print a message or exit code and prompt again.

### Step 4

Now, we will extend both the parser and the executor. 

#### Parsing

```rust
pub fn parse_pipeline(_input: &str) -> Result<Pipeline, ParseError>
```

`parse_pipeline` splits `_input` on `|` and parses each individual
part. For instance, `"ls -la | wc -l"` becomes two `Command`s. Treat
incomplete inputs like `ls |`, `| ls`, and `a | | b` as
`ParseError::EmptyStage`.

#### Execution

Now we will connect the stages. Every stage except the last one sends
its stdout into a `Stdio::piped()`, and the next stage reads that
through `Stdio::from(prev_stdout)`. The last stage then inherits the
shell's stdout. Make sure to wait for all of the children, not just
the last.

>  Note: the classic bug here is a pipeline that just hangs. Give a
> stage a `piped()` stdout only when there is a next stage to read it,
> and make sure you `take()` the child's stdout so it is handed to
> exactly one reader.

### Step 5

Now we want to teach the parser to recognize redirection operators
inside a stage and teach the executor to obey them.

- `< file` sets `Command.stdin` to that file name.
- `> file` sets `Command.stdout` to `(file, RedirectMode::Truncate)`.
- `>> file` sets `Command.stdout` to `(file, RedirectMode::Append)`.
- an operator with no file after it is `ParseError::MissingRedirectTarget`.

In `run_pipeline`, a stage's own redirection wins over the pipe. 

- If `stage.stdin` is set, open that file for input.
- If `stage.stdout` is set, open or create that file for output,
  truncating or appending to match the mode.

## Step 6 (5416 only)

We will add one flourish to the mini-shell: input redirection via web
requests.  Anyone can do this, but it only counts toward the grade for
those in 5416.

For this task, when processing input redirection in the pipeline
management, you should interpret any filename beginning with `http:`
or `https:` as a URL.  For example, to print the HTML for the Rust
home page to the terminal, you should be able tp write.

```sh
cat < https://rust-lang.org
```

We suggest the [reqwest][reqwest] library (using the blocking option
should be fine; we will deal with Rust's async I/O later in the
semester).  You should *not* simply make a local copy; the shell
process should feed the retrieved data into the relevant input pipe
directly.

[reqwest]: https://docs.rs/reqwest/latest/reqwest/

## Rules

1. Keep parsing in the library and execution in the binary. Do not
   spawn processes from `src/lib.rs`.
2. You may use the Rust standard library (`std::process`, `std::fs`,
   `std::env`, `std::io`). Do not hand the work off to a real shell
   (`sh -c`, `system`, or a `popen`-style wrapper).
3. A failing or unknown command must not crash the shell.
