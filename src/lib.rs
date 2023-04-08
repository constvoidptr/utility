//! Collection of often used utilities
//!
//! Every module behind a feature gates of the same name to speed up compilation and reduce dependencies.

#[cfg(feature = "repl")]
pub mod repl;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "telegram")]
pub mod telegram;
