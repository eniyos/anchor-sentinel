//! Report generators for the CLI.
//!
//! `text` is the human-readable output.
//! `json` and `sarif` are machine-readable, byte-stable formats used
//! by CI — they must remain free of ANSI codes and animations.
//! `explain` provides detailed rule explanations for educational use.

pub mod explain;
pub mod json;
pub mod sarif;
pub mod text;
pub mod tty;
