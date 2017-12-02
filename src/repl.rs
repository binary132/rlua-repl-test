extern crate rlua;
extern crate rustyline;

use std::io::Cursor;

use super::{MultiLine, lua};

use morpha::Morpha;

use rustyline::error::ReadlineError;
use rustyline::history::History;

use rlua::Lua;

/// Operate on the given line, using either the given Morpha parser or
/// Lua vm, as appropriate; if the line begins with :l, then the Lua vm
/// will be operated on.
///
/// If the Lua vm or Morpha parser requests another line of input, the
/// Ok result will be a MultiLine::More; otherwise, it will be
/// MultiLine::Done.
///
/// The bool return value indicates whether the given line was a Lua
/// command, so that the caller will know to pass was_lua=true for a
/// MultiLine::More Result.
///
/// Passing was_lua=true causes the line to be interpreted as a Lua
/// command, for multi-line input cases.
pub fn next(
    m: &Morpha,
    vm: &Lua,
    line: Result<String, ReadlineError>,
    hist: &mut History,
    accum: &str,
    count: u32,
    was_lua: bool,
) -> (Result<MultiLine, String>, bool) {

    match line {
        // Got a line of input.
        Ok(line) => {
            hist.add(&line);
            if was_lua {
                // Currently evaluating a Lua block, with accum.
                (
                    lua::attempt_lua(&vm, &[accum, &line].join("\n  "), count),
                    true,
                )
            } else if line.starts_with(":l") {
                // Begin a Lua block.
                (lua::attempt_lua(&vm, line.split_at(2).1, count), true)
            } else {
                // It was a Morpha composition.
                let c = Cursor::new(line);
                (
                    Ok(MultiLine::Done(format!("{:?}", m.lex(c).next().unwrap()))),
                    false,
                )
            }
        }

        // EoF.  Finish the current block if any.
        Err(ReadlineError::Eof) => {
            if accum.len() == 0 {
                (Err(format!("C-d")), false)
            } else if was_lua {
                // Try to evaluate what we have so far, but if it asks
                // for more then finish up.
                match lua::attempt_lua(&vm, accum, count) {
                    Ok(MultiLine::More(_)) => (
                        Ok(MultiLine::Done(String::from("incomplete input"))),
                        false,
                    ),

                    // Otherwise return whatever came back.
                    v => (v, false),
                }
            } else {
                (Ok(MultiLine::Done(String::from("incomplete input"))), false)
            }
        }

        // Interrupt.
        Err(ReadlineError::Interrupted) => (Err(format!("C-c")), false),

        // Something went wrong.
        Err(error) => (Err(format!("Something went wrong: {}", error)), false),
    }
}
