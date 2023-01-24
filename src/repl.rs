//! Define a REPL
//!
//! # Example
//! ```rust
//! use utility::repl::prelude::*;
//!
//! repl::<Commands>();
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
//! impl Evaluate for Commands {
//!     fn evaluate(&mut self) -> ControlFlow {
//!         match self {
//!             Self::Add(person) => println!("Added person: {person:?}"),
//!             Self::Remove(person) => println!("Removed person: {person:?}"),
//!             Self::Exit => return ControlFlow::Exit,
//!         }
//!
//!         ControlFlow::Continue
//!     }
//! }
//! ```
use std::io::Write;

/// Types needed for the REPL definition
pub mod prelude {
    pub use super::repl;
    pub use super::ControlFlow;
    pub use super::Evaluate;
}

/// Indicates if the REPL loop should quit
pub enum ControlFlow {
    Continue,
    Exit,
}

/// Definition for the evaluate function
pub trait Evaluate: clap::Parser {
    /// Evaluation of the parsed input
    ///
    /// Return [`ControlFlow::Exit`] if you wish to exit the loop
    fn evaluate(&mut self) -> ControlFlow;
}

/// Run the REPL
///
/// See top level documentation for an example
pub fn repl<E: Evaluate>() {
    let mut control_flow = ControlFlow::Continue;
    let mut buf = String::new();
    let mut parser: Option<E> = None;

    while !matches!(control_flow, ControlFlow::Exit) {
        let Some(words) = read(&mut buf) else {
            println!("error: malformed input");
            continue;
        };

        if words.is_empty() {
            continue;
        }

        // Create a parser instance or update if it already exists
        let update = match &mut parser {
            Some(p) => p.try_update_from(&words).map(|_| p),
            None => E::try_parse_from(words).map(|p| parser.insert(p)),
        };

        let parser = match update {
            Ok(p) => p,
            Err(err) => {
                println!("{err}");
                continue;
            }
        };

        control_flow = parser.evaluate();
    }
}

fn read(buf: &mut String) -> Option<Vec<String>> {
    print!("{}> ", env!("CARGO_PKG_NAME"));
    std::io::stdout().flush().expect("failed to flush stdout");

    buf.clear();
    std::io::stdin()
        .read_line(buf)
        .expect("failed to read line from stdin");

    shlex::split(buf.trim())
}
