//! TTY / CI / NO_COLOR detection.
//!
//! Every animation and color call in the report layer should be gated on
//! `interactive()` so the output is plain when stdout is piped (which is
//! how the test suite runs), when running in CI, or when the user has
//! explicitly opted out of color.
//!
//! Precedence (most restrictive wins):
//!   1. `NO_COLOR` set to any non-empty value       → no color, no animation
//!   2. `CI=true` or `TERM=dumb`                     → no animation (still
//!      color, since a CI log might want it for SARIF-less review)
//!   3. `is_terminal::IsTerminal::is_terminal(stdout)` → no color, no animation
//!
//! The intent is that the rules engine and JSON/SARIF output are completely
//! unaffected by any of this — they never call into the report layer.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

/// Cached answer to `interactive()`. Computed once on first call so
/// the spinner thread (which polls it on every frame) doesn't keep
/// hitting env vars.
static INTERACTIVE: AtomicBool = AtomicBool::new(false);
static COMPUTED: AtomicBool = AtomicBool::new(false);

/// True iff the report layer may use color, animation, and the spinner.
pub fn interactive() -> bool {
    if COMPUTED.load(Ordering::Relaxed) {
        return INTERACTIVE.load(Ordering::Relaxed);
    }
    let value = compute();
    INTERACTIVE.store(value, Ordering::Relaxed);
    COMPUTED.store(true, Ordering::Relaxed);
    value
}

/// Reset the cached answer. Test-only.
#[cfg(test)]
#[allow(dead_code)]
pub fn reset_for_tests() {
    COMPUTED.store(false, Ordering::Relaxed);
    INTERACTIVE.store(false, Ordering::Relaxed);
}

fn compute() -> bool {
    // 1. NO_COLOR: any non-empty value disables color. We treat that
    //    as a strong signal — also disable animation. The convention
    //    is https://no-color.org.
    if let Ok(v) = std::env::var("NO_COLOR") {
        if !v.is_empty() {
            return false;
        }
    }
    // 2. CI: most CI providers set CI=true. Also TERM=dumb for dumb
    //    terminals. We allow color in plain CI (the SARIF-less
    //    review workflow benefits from it) but skip animations that
    //    would interfere with log scraping.
    if std::env::var("CI").ok().as_deref() == Some("true")
        || std::env::var("TERM").ok().as_deref() == Some("dumb")
    {
        return false;
    }
    // 3. Actually a terminal? When stdout is piped (the test suite
    //    pipes it), this is false, and we get plain output.
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piping_stdout_disables_color() {
        // The test suite always runs with stdout piped, so `interactive()`
        // should return false here.
        assert!(!interactive());
    }
}
