//! Plugin-style rule registry built on `inventory`.
//!
//! Each rule file in `src/rules/*.rs` submits a factory via
//! `inventory::submit!`. `all_rules()` instantiates them on demand.
//!
//! The `RuleFactory` is `fn() -> Box<dyn Rule>` so the actual rule struct
//! can carry its own state (currently none needed, but it leaves the door
//! open for rule configuration later).

use std::sync::Arc;

use super::Rule;

/// Submit a rule into the global registry.
pub struct RuleFactory {
    pub build: fn() -> Arc<dyn Rule>,
}

inventory::collect!(RuleFactory);

/// Instantiate every registered rule, in registration order. Order is
/// deterministic per build but not guaranteed across crates.
pub fn all_rules() -> Vec<Arc<dyn Rule>> {
    let mut out: Vec<Arc<dyn Rule>> = Vec::new();
    for f in inventory::iter::<RuleFactory> {
        out.push((f.build)());
    }
    out
}

/// Iterate rule metadata without instantiating. Useful for `sentinel rules`.
pub fn list_rule_ids() -> Vec<(&'static str, Severity, &'static str)> {
    all_rules()
        .into_iter()
        .map(|r| (r.id(), r.severity(), r.description()))
        .collect()
}

// Re-export the Severity enum so the `list_rule_ids` return type works.
use super::Severity;
