//! Collection of utilities
//!
//! Every module is behind a feature gates of the same name.

#[cfg(feature = "repl")]
pub mod repl;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "telegram")]
pub mod telegram;

#[cfg(all(feature = "tts", target_os = "windows"))]
pub mod tts;
