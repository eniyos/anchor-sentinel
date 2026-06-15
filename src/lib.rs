//! Anchor Sentinel core library.
//!
//! This crate provides the rule engine, IDL parser, AST parser, and report
//! generators for the `sentinel` CLI binary (`src/main.rs`).

//! Modules:
//!   - [`ast`]: Rust source visitor that produces `AstHint`s for the engine.
//!   - [`config`]: config file (`sentinel.toml`) loading and path exclusion.
//!   - [`engine`]: the `Rule` trait, `AnalysisContext`, `Finding`, severity.
//!   - [`idl`]: IDL parser producing the unified `ProgramIr`.
//!   - [`report`]: text / JSON / SARIF report generators.
//!   - [`rules`]: the 14 built-in security rules.

pub mod ast;
pub mod config;
pub mod engine;
pub mod idl;
pub mod report;
pub mod rules;
