extern crate rustyline;
extern crate rlua;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::History;

use rlua::Lua;

/// MultiLine represents the result of an eval: either it was done (and
/// has a String representation), or it wanted more input (and has the
/// current accumulated input.)
#[derive(Debug)]
enum MultiLine {
    More(String),
    Done(String),
}

fn attempt(some: &Lua, run: &str, next: u32) -> Result<MultiLine, String> {
    // println!("{}", run);
    match some.eval::<rlua::MultiValue>(run, Some(&format!("{}", next))) {
        // Ok(rlua::MultiValue(values)) => Ok(MultiLine::Done(format!("{:?}", values))),

        // Evaluated OK.
        Ok(v) => {
            let it = v.into_iter();
            Ok(MultiLine::Done(if it.len() == 0 {
                format!("")
            } else {
                it.map(|next| format!("{:?}\t", next)).collect()
            }))
        }

        // Syntax error.
        Err(rlua::Error::SyntaxError {
                message: m,
                incomplete_input: false,
            }) => Ok(MultiLine::Done(format!("Lua syntax error: {}", m))),

        // Syntax error because input is incomplete, ask for more.
        Err(rlua::Error::SyntaxError { incomplete_input: true, .. }) => Ok(MultiLine::More(
            String::from(run),
        )),

        Err(rlua::Error::RuntimeError(e)) => Ok(MultiLine::Done(format!("{}", e))),

        // Some other problem.
        Err(err) => Err(format!("Error in eval: {:?}", err)),
    }
}

fn next(
    vm: &Lua,
    line: Result<String, ReadlineError>,
    hist: &mut History,
    accum: &str,
    count: u32,
) -> Result<MultiLine, String> {
    match line {

        // Got a line of input.
        Ok(line) => {
            hist.add(&line);
            if accum.len() == 0 {
                // Try to evaluate it.
                attempt(&vm, &line, count)
            } else {
                // Eval accum + line.
                attempt(&vm, &[accum, &line].join("\n  "), count)
            }
            // Is this a Lua command?
            // if line.starts_with(":l") {
            //     let line = line.split_at(2).1;
            //     if accum.len() == 0 {
            //         // Try to evaluate it.
            //         attempt(&vm, line, count)
            //     } else {
            //         // Eval accum + line.
            //         attempt(&vm, &[accum, line].join("\n  "), count)
            //     }
            // } else {
            //     Ok(MultiLine::Done(String::from("")))
            // }
        }

        // EoF.  Finish the current block if any.
        Err(ReadlineError::Eof) => {
            if accum.len() == 0 {
                Err(format!("C-d"))
            } else {
                match attempt(&vm, accum, count) {
                    // Try to evaluate what we have so far, but if it asks
                    // for more then finish up.
                    Ok(MultiLine::More(_)) => Ok(MultiLine::Done(String::from("incomplete input"))),

                    // Otherwise return whatever came back.
                    v => v,
                }
            }
        }

        // Interrupt.
        Err(ReadlineError::Interrupted) => Err(format!("C-c")),

        // Something went wrong.
        Err(error) => Err(format!("Something went wrong: {}", error)),
    }
}

const HISTFILE: &str = ".repl_history";

fn main() {
    let mut ed = Editor::<()>::new();
    let _ = ed.load_history(HISTFILE);

    let vm = Lua::new();

    let mut count = 0;
    let mut accum = String::from("");
    let mut prompt = "Λ ";

    loop {
        match next(&vm, ed.readline(prompt), ed.get_history(), &accum, count) {
            Ok(MultiLine::More(body)) => {
                // Accumulate more input with no prompt.
                prompt = "    ";
                accum = body;
            }

            Ok(MultiLine::Done(result)) => {
                // Finished evaluating something, display it.
                println!("{}", result);
                accum.clear();
                prompt = "Λ ";
                count += 1;
            }

            Err(err) => {
                // Something went wrong.
                println!("{}", err);
                break;
            }
        }
    }

    if let Err(e) = ed.save_history(HISTFILE) {
        println!("Failed to save history: {:?}", e)
    }
}
