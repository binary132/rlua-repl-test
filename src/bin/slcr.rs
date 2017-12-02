extern crate slcr;

extern crate morpha;

extern crate rustyline;
extern crate rlua;

use slcr::{loader, repl, MultiLine};
use morpha::Morpha;

use rustyline::Editor;
use rlua::Lua;

const HISTFILE: &str = ".repl_history";

fn main() {
    let mut ed = Editor::<()>::new();
    let _ = ed.load_history(HISTFILE);

    let vm = Lua::new();

    // Load the libraries.
    if let Err(e) = loader::load_all(&vm) {
        println!("failed to load: {}", e);
    }

    let mut count = 0;
    let mut accum = String::from("");
    let mut prompt = "Λ ";
    let mut was_lua = false;
    let morpha = Morpha::new();

    loop {
        let result = repl::next(
            &morpha,
            &vm,
            ed.readline(prompt),
            ed.get_history(),
            &accum,
            count,
            was_lua,
        );

        was_lua = result.1;

        match result.0 {
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
                was_lua = false;
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
