//! CLI argument definitions.

use clap::{Parser, Subcommand, ValueEnum};

use crate::engine::Severity;

#[derive(Parser, Debug)]
#[command(
    name = "sentinel",
    version,
    about = "Static security analysis for Anchor programs"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan an Anchor project for security issues.
    Scan {
        /// Path to the Anchor project (the directory containing `Anchor.toml`).
        path: String,

        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Treat any non-info finding as a build failure. Useful in CI.
        #[arg(long)]
        strict: bool,

        /// Comma-separated list of rule ids to skip.
        #[arg(long, value_delimiter = ',')]
        ignore: Vec<String>,

        /// Minimum severity to report. Lower-severity findings are hidden.
        /// Exit code is 1 when any finding at or above this severity exists.
        #[arg(long, value_enum)]
        min_severity: Option<MinSeverity>,

        /// Show progress and timing for each scan phase.
        #[arg(short, long)]
        verbose: bool,
    },

    /// List all registered rules.
    Rules,

    /// Print the sentinel version.
    Version,

    /// Explain a security rule in detail.
    Explain {
        /// The rule id to explain (e.g., missing_signer).
        rule_id: String,
    },
}

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum MinSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl MinSeverity {
    pub fn into_severity(self) -> Severity {
        match self {
            MinSeverity::Info => Severity::Info,
            MinSeverity::Low => Severity::Low,
            MinSeverity::Medium => Severity::Medium,
            MinSeverity::High => Severity::High,
            MinSeverity::Critical => Severity::Critical,
        }
    }
}
