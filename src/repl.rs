//! Define a REPL
//!
//! # Example
//! ```rust
//! use utility::repl::prelude::*;
//!
//! repl(evaluate);
//!
//! #[derive(clap::Args, Debug)]
//! struct Person {
//!     /// Name of the person
//!     name: String,
//!     /// Age of the person
//!     age: u32,
//! }
//!
//! #[derive(clap::Parser)]
//! #[command(multicall = true)]
//! enum Commands {
//!     /// Add a `person` to the database
//!     Add(Person),
//!     /// Remove a `person` from the database
//!     Remove(Person),
//!     /// Exit the program
//!     #[clap(visible_alias = "quit")]
//!     Exit,
//! }
//!
//! fn evaluate(commands: &mut Commands) -> ControlFlow {
//!     match commands {
//!         Commands::Add(person) => println!("Added person: {person:?}"),
//!         Commands::Remove(person) => println!("Removed person: {person:?}"),
//!         Commands::Exit => return ControlFlow::Exit,
//!     }
//!     ControlFlow::Continue
//! }
//! ```
use std::io::Write;

/// Types needed for the REPL definition
pub mod prelude {
    pub use super::repl;
    pub use super::ControlFlow;
}

/// Indicates if the REPL loop should quit
pub enum ControlFlow {
    Continue,
    Exit,
}

/// Run the REPL
///
/// See top level documentation for an example
pub fn repl<P, F>(mut evaluate: F)
where
    P: clap::Parser,
    F: FnMut(&mut P) -> ControlFlow,
{
    let mut control_flow = ControlFlow::Continue;
    let mut buf = String::new();
    let mut parser: Option<P> = None;

    while !matches!(control_flow, ControlFlow::Exit) {
        let Some(words) = read(&mut buf) else {
            println!("error: malformed input");
            continue;
        };

        if words.is_empty() {
            continue;
        }

        let parser = match P::try_parse_from(words) {
            Ok(p) => parser.insert(p),
            Err(err) => {
                println!("{err}");
                continue;
            }
        };

        control_flow = evaluate(parser);
    }
}

fn read(buf: &mut String) -> Option<Vec<String>> {
    print!("> ");
    std::io::stdout().flush().expect("failed to flush stdout");

    buf.clear();
    std::io::stdin()
        .read_line(buf)
        .expect("failed to read line from stdin");

    shlex::split(buf.trim())
}
