use std::env;
use std::io::{self, Write, BufRead, Result};
use std::path::Path;
use std::process::{Child, ChildStdout, Command};

use minishell::{as_builtin, parse_pipeline, Builtin, Pipeline};

/// Wrapper around read_line for Stdin and for file BufReader instances
trait LineReader {
    fn read(&mut self, buf: &mut String) -> Result<usize>;
}

/// Wrapper around read_line for Stdin (usual command source)
impl LineReader for io::Stdin {
    fn read(&mut self, buf: &mut String) -> Result<usize> {
        self.read_line(buf)
    }
}

/// Wrapper around read_line for a file wrapped in a BufReader
impl LineReader for io::BufReader<std::fs::File> {
    fn read(&mut self, buf: &mut String) -> Result<usize> {
        self.read_line(buf)
    }
}

/// Write a prompt for the interactive mini-shell
fn write_prompt() -> io::Result<()> {
    let prompt = match env::current_dir() {
        Ok(dir) => format!("minishell:{}> ", dir.display()),
        Err(_) => "minishell> ".to_string(),
    };
    print!("{prompt}");
    io::stdout().flush() // Ensure prompt shows before input
}

/// 

/// Mini-shell REPL
fn repl<T: LineReader>(is_interactive: bool, mut cmd_stream: T) {
    loop {
        // Produce prompt if we're in interactive mode
        if is_interactive && write_prompt().is_err() {
            break;
        }

        let mut line = String::new();
        match cmd_stream.read(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("minishell: read error: {e}");
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let pipeline = match parse_pipeline(line) {
            Ok(pipeline) => pipeline,
            Err(err) => {
                eprintln!("minishell: parse error: {err:?}");
                continue;
            }
        };

        if pipeline.stages.len() == 1 {
            let stage = &pipeline.stages[0];
            match as_builtin(&stage.program) {
                Some(Builtin::Exit) => return,
                Some(Builtin::Cd) => {
                    todo!("Change the shell's cwd here (cd) ");
                }
                Some(Builtin::Pwd) => {
                    match env::current_dir() {
                        Ok(dir) => println!("{}", dir.display()),
                        Err(e) => eprintln!("pwd: {e}"),
                    }
                    continue;
                }
                Some(Builtin::Source) => {
                    todo!("Read and execute commands from file here (source)");
                }
                None => {} // Not a built-in: fall through to external execution.
            }
        }
        if let Err(e) = run_pipeline(&pipeline) {
            eprintln!("minishell: {e}");
        }
    }
}

// Execute an external Pipeline and return the exit code of the last stage
fn run_pipeline(pipeline: &Pipeline) -> io::Result<i32> {
    // Output of the previous stage
    let mut prev_stdout: Option<ChildStdout> = None;
    // All spawned children,  we can wait on them after the loop.
    let mut children: Vec<Child> = Vec::new();

    let mut stages = pipeline.stages.iter().peekable();
    while let Some(stage) = stages.next() {
        let is_last = stages.peek().is_none();

        let mut cmd = Command::new(&stage.program);
        cmd.args(&stage.args);

        // Recommended strategy
        // 1: decide this stage's stdin, then call cmd.stdin(...)
        // - step 4: does it come from the previous stage or the
        //   shell's own input?
        // - step 5: what if the stage names its own file with `<`?
        // - step 6: what if the stage specifies `< URL`?

        // 2: decide this stage's stdout, then call cmd.stdout(...)
        // - step 4: does it need to go to the next stage, or should
        //   the user see it?
        // - step 5: what if the stage names its own file with `>` / `>>`?

        // 3: spawn the command and keep track of it
        // - on success: hand this stage's output to the next stage,
        //   and remember the child
        // - on failure: what should happen to the rest of the pipeline?

        // This line only exists so the starter compiles - delete it once 1-3 are done.
        let _ = (is_last, &mut cmd, &mut prev_stdout, &mut children);
        todo!("Step 3: spawn + track child; step 4: wire pipes; step 5: apply redirection");
    }
    // Step 3: wait for every child to finish. Return the exit code of the last child.
    todo!("Step 3: wait for all children and return the final stage's exit code");
}

fn main() {
    repl(true, io::stdin());
}
