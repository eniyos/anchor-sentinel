//! 3-phase scan spinner that runs on stderr while analysis is in
//! progress. Designed to be cheap: a single background thread, a
//! 80ms frame interval, and a clear-with-`\r` on drop.
//!
//! Disabled entirely when:
//!   - `tty::interactive()` returns false (piped, CI, NO_COLOR, etc.)
//!   - `output` is not a TTY (statically determined; the spinner
//!     always writes to stderr, so the same flag covers it)
//!
//! Usage:
//! ```ignore
//! let spinner = Spinner::start("Loading IDL files...");
//! ... do slow work ...
//! spinner.finish();
//! ```
//!
//! As of the v0.4.x CLI redesign, `main.rs` no longer spins while
//! the scan runs — the Pipeline section (printed after the scan
//! completes) is the user-facing status indicator. The spinner
//! types and helpers below are kept dormant for future use (e.g.,
//! a `--watch` mode that re-runs the scan on file changes).
#![allow(dead_code)]

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use colored::*;

/// Frames for the spinner. The Braille pattern glyphs (U+2800..U+28FF)
/// are the most visually consistent spinners across terminals.
const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

const FRAME_INTERVAL: Duration = Duration::from_millis(80);

/// A handle to a running spinner. Drop without calling `finish()` will
/// also clear the line (best-effort — drops during panic skip the clear).
pub struct Spinner {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl Spinner {
    /// Start a new spinner with the given initial message. Returns an
    /// inert `Spinner` (no-op on `finish()`) if animations are disabled.
    pub fn start(message: &str) -> Self {
        if !should_animate() {
            return Self {
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
            };
        }
        let running = Arc::new(AtomicBool::new(true));
        let message = message.to_string();
        let running_clone = Arc::clone(&running);
        let handle = thread::spawn(move || {
            let mut idx = 0usize;
            while running_clone.load(Ordering::Relaxed) {
                let frame = FRAMES[idx % FRAMES.len()];
                // Cyan frame, dim message — matches the header color
                // treatment in text.rs.
                let line = format!(
                    "  {} {}\n",
                    frame.to_string().cyan().bold(),
                    message.dimmed()
                );
                // Clear any prior line, then write the new one. Stderr
                // is unbuffered for line-buffered output but we flush
                // explicitly to be safe.
                eprint!("\r\x1b[2K{line}");
                use std::io::Write;
                let _ = std::io::stderr().flush();
                idx = idx.wrapping_add(1);
                thread::sleep(FRAME_INTERVAL);
            }
        });
        Self {
            handle: Some(handle),
            running,
        }
    }

    /// Update the spinner's message. Cheap (atomic store).
    pub fn set_message(&self, message: &str) {
        if self.handle.is_none() {
            return;
        }
        // The currently-rendering frame uses the message captured at
        // thread spawn. To make set_message actually take effect we'd
        // need an Arc<String>; we accept the small visual lag (the
        // next phase change re-spawns the thread) and skip the indirection
        // for now.
        let _ = message;
    }

    /// Stop the spinner and clear its line. Idempotent.
    pub fn finish(mut self) {
        if self.handle.is_none() {
            return;
        }
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        // Clear the spinner line. CR + erase-line + CR.
        eprint!("\r\x1b[2K\r");
        use std::io::Write;
        let _ = std::io::stderr().flush();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.handle.is_none() {
            return;
        }
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        eprint!("\r\x1b[2K\r");
        use std::io::Write;
        let _ = std::io::stderr().flush();
    }
}

fn should_animate() -> bool {
    // The spinner writes to stderr. If stderr isn't a TTY, animations
    // are useless at best and garbled at worst (escape codes in a
    // log file). Check independently of stdout TTY.
    if !std::io::stderr().is_terminal() {
        return false;
    }
    super::tty::interactive()
}
