//! Report generators for the CLI.
//!
//! `text` is the human-readable, possibly-animated output.
//! `json` and `sarif` are machine-readable, byte-stable formats used
//! by CI — they must remain free of ANSI codes and animations.

pub mod json;
pub mod sarif;
pub mod spinner;
pub mod text;
pub mod tty;
