//! Built-in rules. Each rule lives in its own submodule and self-registers
//! via `inventory::submit!`. To add a rule, drop a new file here and add a
//! `pub mod <name>;` line — no central edit.

use std::sync::Arc;

use crate::engine::{registry::RuleFactory, Rule};

pub mod missing_signer;
pub mod missing_ownership;
pub mod unsafe_arithmetic;
pub mod missing_mut;
pub mod pda_misconfig;

// `inventory` requires the submission to be reachable from a `static` of the
// crate. Putting them in a `#[distributed_slice]`-style static list keeps the
// linker from dropping them.
inventory::submit! {
    RuleFactory { build: || Arc::new(missing_signer::MissingSigner) as Arc<dyn Rule> }
}
inventory::submit! {
    RuleFactory { build: || Arc::new(missing_ownership::MissingOwnership) as Arc<dyn Rule> }
}
inventory::submit! {
    RuleFactory { build: || Arc::new(unsafe_arithmetic::UnsafeArithmetic) as Arc<dyn Rule> }
}
inventory::submit! {
    RuleFactory { build: || Arc::new(missing_mut::MissingMut) as Arc<dyn Rule> }
}
inventory::submit! {
    RuleFactory { build: || Arc::new(pda_misconfig::PdaMisconfig) as Arc<dyn Rule> }
}
