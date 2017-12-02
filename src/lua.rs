use super::MultiLine;

use rlua::{Lua, MultiValue, Error};
use std::str;

pub fn attempt_lua(some: &Lua, run: &str, next: u32) -> Result<MultiLine, String> {
    match some.eval::<MultiValue>(run, Some(&format!("{}", next))) {
        // Evaluated OK.
        Ok(v) => {
            let it = v.into_iter();
            Ok(MultiLine::Done(if it.len() == 0 {
                format!("")
            } else {
                it.map(|next| match next {
                    // rlua::Value::String(s) => format!("{:?}\t", s),
                    n => format!("{:?}\t", n),
                }).collect()
            }))
        }

        // Syntax error.
        Err(Error::SyntaxError {
                message: m,
                incomplete_input: false,
            }) => Ok(MultiLine::Done(format!("Lua syntax error: {}", m))),

        // Syntax error because input is incomplete, ask for more.
        Err(Error::SyntaxError { incomplete_input: true, .. }) => Ok(MultiLine::More(
            String::from(run),
        )),

        Err(Error::RuntimeError(e)) => Ok(MultiLine::Done(format!("{}", e))),

        // Some other problem.
        Err(err) => Err(format!("Error in eval: {:?}", err)),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_attempt_lua() {
        assert_eq!(2, 2);
    }
}
